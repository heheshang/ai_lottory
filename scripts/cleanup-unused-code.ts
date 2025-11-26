/**
 * Unused Code Cleanup Script
 *
 * Analyzes the codebase to identify and remove:
 * - Unused imports
 * - Unused functions and variables
 * - Dead code
 * - Unused dependencies
 * - Duplicate code
 * - Commented-out code blocks
 */

import * as fs from 'fs'
import * as path from 'path'
import { glob } from 'glob'

interface CleanupReport {
  unusedImports: Array<{
    file: string
    imports: string[]
  }>
  unusedExports: Array<{
    file: string
    exports: string[]
  }>
  deadCode: Array<{
    file: string
    functions: string[]
    variables: string[]
  }>
  duplicateCode: Array<{
    file: string
    duplicates: string[]
  }>
  unusedDependencies: string[]
  totalFiles: number
  totalLines: number
  linesRemoved: number
  filesModified: number
}

interface ImportAnalysis {
  imports: Set<string>
  used: Set<string>
  unused: Set<string>
}

interface CodeAnalysis {
  functions: Set<string>
  variables: Set<string>
  used: Set<string>
  unused: Set<string>
}

class CodeCleanupAnalyzer {
  private report: CleanupReport = {
    unusedImports: [],
    unusedExports: [],
    deadCode: [],
    duplicateCode: [],
    unusedDependencies: [],
    totalFiles: 0,
    totalLines: 0,
    linesRemoved: 0,
    filesModified: 0
  }

  private importMap = new Map<string, ImportAnalysis>()
  private codeMap = new Map<string, CodeAnalysis>()

  async analyzeProject(rootDir: string): Promise<CleanupReport> {
    console.log('🔍 Analyzing project for unused code...')

    // Find all TypeScript and Vue files
    const files = await this.getAllFiles(rootDir)
    this.report.totalFiles = files.length

    console.log(`📁 Found ${files.length} files to analyze`)

    // Analyze each file
    for (const file of files) {
      await this.analyzeFile(file)
    }

    // Analyze package.json for unused dependencies
    await this.analyzeDependencies(rootDir)

    // Find duplicate code blocks
    await this.findDuplicateCode(rootDir)

    // Generate cleanup suggestions
    await this.generateCleanupSuggestions()

    console.log('✅ Analysis complete!')
    return this.report
  }

  private async getAllFiles(rootDir: string): Promise<string[]> {
    const patterns = [
      '**/*.ts',
      '**/*.vue',
      '**/*.js'
    ]

    const files: string[] = []

    for (const pattern of patterns) {
      const matched = glob.sync(pattern, {
        cwd: rootDir,
        ignore: [
          '**/node_modules/**',
          '**/dist/**',
          '**/build/**',
          '**/.git/**',
          '**/coverage/**',
          '**/*.d.ts'
        ]
      })

      files.push(...matched.map(file => path.join(rootDir, file)))
    }

    return files.sort()
  }

  private async analyzeFile(filePath: string): Promise<void> {
    const content = fs.readFileSync(filePath, 'utf-8')
    const lines = content.split('\n')
    this.report.totalLines += lines.length

    // Analyze imports
    const importAnalysis = this.analyzeImports(content)
    this.importMap.set(filePath, importAnalysis)

    // Analyze code usage
    const codeAnalysis = this.analyzeCodeUsage(content)
    this.codeMap.set(filePath, codeAnalysis)

    // Find unused imports
    const unusedImports = this.findUnusedImports(importAnalysis, codeAnalysis)
    if (unusedImports.length > 0) {
      this.report.unusedImports.push({
        file: path.relative(process.cwd(), filePath),
        imports: unusedImports
      })
    }

    // Find dead code
    const deadCode = this.findDeadCode(codeAnalysis, content)
    if (deadCode.functions.length > 0 || deadCode.variables.length > 0) {
      this.report.deadCode.push({
        file: path.relative(process.cwd(), filePath),
        functions: deadCode.functions,
        variables: deadCode.variables
      })
    }

    // Find commented-out code blocks
    const commentedCode = this.findCommentedCode(content)
    if (commentedCode.length > 0) {
      console.log(`📝 Found ${commentedCode.length} commented code blocks in ${path.basename(filePath)}`)
    }
  }

  private analyzeImports(content: string): ImportAnalysis {
    const imports = new Set<string>()
    const used = new Set<string>()
    const unused = new Set<string>()

    // Match various import patterns
    const importPatterns = [
      // import { ... } from '...'
      /import\s*{([^}]+)}\s*from\s*['"]([^'"]+)['"]/g,
      // import ... from '...'
      /import\s*(\w+)\s*from\s*['"]([^'"]+)['"]/g,
      // import '...'
      /import\s*['"]([^'"]+)['"]/g,
      // const ... = require('...')
      /const\s*(\w+)\s*=\s*require\(['"])([^'"]+)(['"])/g,
      // @import('...')
      /@import\(['"])([^'"]+)(['"])/g,
      // @Directive('...')
      /@Directive\(['"])([^'"]+)(['"])/g,
      // @Injectable('...')
      /@Injectable\(['"])([^'"]+)(['"])/g
    ]

    for (const pattern of importPatterns) {
      let match
      while ((match = pattern.exec(content)) !== null) {
        const importName = match[1] || match[2]
        const moduleName = match[3] || match[4] || match[5]

        if (importName && moduleName) {
          imports.add(`${importName} from ${moduleName}`)
        }
      }
    }

    // Find usage of imported items
    for (const importStr of imports) {
      const importName = this.getImportName(importStr)
      if (importName && content.includes(importName)) {
        used.add(importStr)
      }
    }

    // Determine unused imports
    for (const importStr of imports) {
      if (!used.has(importStr)) {
        unused.add(importStr)
      }
    }

    return { imports, used, unused }
  }

  private analyzeCodeUsage(content: string): CodeAnalysis {
    const functions = new Set<string>()
    const variables = new Set<string>()
    const used = new Set<string>()

    // Find function declarations
    const functionPatterns = [
      // function name() {}
      /function\s+(\w+)\s*\(/g,
      // const name = function() {}
      /const\s+(\w+)\s*=\s*function\s*/g,
      // const name = () => {}
      /const\s+(\w+)\s*=\s*\([^)]*\)\s*=>/g,
      // export function name() {}
      /export\s+function\s+(\w+)\s*\(/g,
      // export const name = () => {}
      /export\s+const\s+(\w+)\s*=\s*([^)]*\)\s*=>/g,
      // class name {}
      /class\s+(\w+)/g,
      // methods
      /(\w+)\s*\([^)]*\)\s*\{/g
    ]

    for (const pattern of functionPatterns) {
      let match
      while ((match = pattern.exec(content)) !== null) {
        functions.add(match[1])
      }
    }

    // Find variable declarations
    const variablePatterns = [
      // const name = ...
      /const\s+(\w+)\s*=/g,
      // let name = ...
      /let\s+(\w+)\s*=/g,
      // var name = ...
      /var\s+(\w+)\s*=/g,
      // interface Name {}
      /interface\s+(\w+)/g,
      // type Name = ...
      /type\s+(\w+)\s*=/g,
      // enum Name {}
      /enum\s+(\w+)/g
    ]

    for (const pattern of variablePatterns) {
      let match
      while ((match = pattern.exec(content)) !== null) {
        variables.add(match[1])
      }
    }

    // Find usage of functions and variables
    for (const name of [...functions, ...variables]) {
      // Simple usage check - look for the name followed by non-word characters
      const usagePattern = new RegExp(`\\b${name}\\b`, 'g')
      if (usagePattern.test(content)) {
        used.add(name)
      }
    }

    // Determine unused functions and variables
    const unusedFunctions = [...functions].filter(name => !used.has(name))
    const unusedVariables = [...variables].filter(name => !used.has(name))

    return {
      functions: new Set(unusedFunctions),
      variables: new Set(unusedVariables),
      used
    }
  }

  private findUnusedImports(importAnalysis: ImportAnalysis, codeAnalysis: CodeAnalysis): string[] {
    const unused: string[] = []

    for (const importStr of importAnalysis.unused) {
      const importName = this.getImportName(importStr)
      if (importName && !codeAnalysis.used.has(importName)) {
        unused.push(importStr)
      }
    }

    return unused
  }

  private findDeadCode(codeAnalysis: CodeAnalysis, content: string): { functions: string[]; variables: string[] } {
    // Check if functions/variables are actually used in component templates, exports, etc.
    const unusedFunctions = [...codeAnalysis.functions].filter(name => {
      // Check if used in template
      const templateMatch = content.includes(`{{${name}}`) ||
                           content.includes(`@${name}`) ||
                           content.includes(`${name}.`) ||
                           content.includes(`this.${name}`)

      return !templateMatch
    })

    const unusedVariables = [...codeAnalysis.variables].filter(name => {
      // Check if used in template or JSX
      const templateMatch = content.includes(`{{${name}}`) ||
                           content.includes(`@${name}`) ||
                           content.includes(`${name}.`) ||
                           content.includes(`this.${name}`)

      return !templateMatch
    })

    return { functions: unusedFunctions, variables: unusedVariables }
  }

  private findCommentedCode(content: string): string[] {
    const commentedBlocks: string[] = []

    // Find multi-line commented blocks
    const multiLineCommentPattern = /\/\*[\s\S]*?\*\/g
    const multiLineMatches = content.match(multiLineCommentPattern)

    if (multiLineMatches) {
      commentedBlocks.push(...multiLineMatches)
    }

    // Find single-line commented code patterns
    const lines = content.split('\n')
    const singleLineCommentPattern = /^\s*\/\/.*?(?:const|let|var|function|import|export|class|interface)\s+\w+/gm

    for (const line of lines) {
      if (singleLineCommentPattern.test(line)) {
        commentedBlocks.push(line.trim())
      }
    }

    return commentedBlocks
  }

  private async analyzeDependencies(rootDir: string): Promise<void> {
    const packageJsonPath = path.join(rootDir, 'package.json')

    if (!fs.existsSync(packageJsonPath)) {
      return
    }

    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'))
    const dependencies = { ...packageJson.dependencies, ...packageJson.devDependencies }

    // Find all used imports across the project
    const usedDependencies = new Set<string>()

    for (const [filePath, importAnalysis] of this.importMap) {
      for (const importStr of importAnalysis.used) {
        const depName = this.extractDependencyName(importStr)
        if (depName && dependencies[depName]) {
          usedDependencies.add(depName)
        }
      }
    }

    // Find unused dependencies
    const unusedDependencies = Object.keys(dependencies).filter(dep => !usedDependencies.has(dep))
    this.report.unusedDependencies = unusedDependencies
  }

  private async findDuplicateCode(rootDir: string): Promise<void> {
    console.log('🔍 Checking for duplicate code blocks...')

    const codeBlocks = new Map<string, string[]>()

    // Find all functions and their implementations
    for (const [filePath, content] of this.codeMap) {
      const functions = this.extractCodeBlocks(content)

      for (const [code, name] of functions) {
        const normalizedCode = this.normalizeCode(code)
        if (!codeBlocks.has(normalizedCode)) {
          codeBlocks.set(normalizedCode, [])
        }
        codeBlocks.get(normalizedCode)!.push(`${path.basename(filePath)}:${name}`)
      }
    }

    // Find duplicates
    for (const [code, locations] of codeBlocks) {
      if (locations.length > 1) {
        this.report.duplicateCode.push({
          file: 'Multiple files',
          duplicates: locations.join(', ')
        })
      }
    }
  }

  private extractCodeBlocks(content: string): Map<string, string> {
    const blocks = new Map<string, string>()

    // Match function definitions
    const functionPattern = /(?:function\s+(\w+)\s*\([^)]*\)\s*\{([^}]*(?:\{[^}]*\}[^}]*)*\})|(?:const\s+(\w+)\s*=\s*function\s*\([^)]*\)\s*\{([^}]*(?:\{[^}]*\}[^}]*)*\})|(?:const\s+(\w+)\s*=\s*\([^)]*\)\s*=>\s*\{([^}]*(?:\{[^}]*\}[^}]*)*\})|(?:export\s+function\s+(\w+)\s*\([^)]*\)\s*\{([^}]*)*\})|(?:export\s+const\s+(\w+)\s*=\s*\([^)]*\)\s*=>\s*\{([^}]*\})/g

    let match
    while ((match = functionPattern.exec(content)) !== null) {
      const functionName = match[1] || match[2] || match[3] || match[4] || match[5]
      const functionBody = match[6] || match[7] || match[8] || match[9] || match[10] || ''

      if (functionName && functionBody.length > 10) { // Ignore very short functions
        blocks.set(functionBody.trim(), functionName)
      }
    }

    return blocks
  }

  private normalizeCode(code: string): string {
    // Normalize code by removing whitespace, comments, and variable names
    return code
      .replace(/\s+/g, ' ') // Multiple spaces to single space
      .replace(/\/\/.*$/gm, '') // Remove single line comments
      .replace(/\/\*[\s\S]*?\*\//g, '') // Remove multi-line comments
      .replace(/\b[a-zA-Z_$][a-zA-Z0-9_$]*\b/g, 'VAR') // Replace variable names
      .trim()
      .toLowerCase()
  }

  private async generateCleanupSuggestions(): Promise<void> {
    console.log('\n📊 Cleanup Report:')
    console.log('==================')

    if (this.report.unusedImports.length > 0) {
      console.log(`\n📦 Unused Imports (${this.report.unusedImports.length} files):`)
      for (const item of this.report.unusedImports) {
        console.log(`  ${item.file}: ${item.imports.length} unused imports`)
        item.imports.forEach(imp => console.log(`    - ${imp}`))
      }
    }

    if (this.report.deadCode.length > 0) {
      console.log(`\n💀 Dead Code (${this.report.deadCode.length} files):`)
      for (const item of this.report.deadCode) {
        console.log(`  ${item.file}:`)
        if (item.functions.length > 0) {
          console.log(`    Functions: ${item.functions.join(', ')}`)
        }
        if (item.variables.length > 0) {
          console.log(`    Variables: ${item.variables.join(', ')}`)
        }
      }
    }

    if (this.report.unusedDependencies.length > 0) {
      console.log(`\n📦 Unused Dependencies (${this.report.unusedDependencies.length}):`)
      this.report.unusedDependencies.forEach(dep => {
        console.log(`  - ${dep}`)
      })
    }

    if (this.report.duplicateCode.length > 0) {
      console.log(`\n🔄 Duplicate Code (${this.report.duplicateCode.length} blocks):`)
      this.report.duplicateCode.forEach(item => {
        console.log(`  ${item.duplicates}`)
      })
    }

    // Generate cleanup script
    await this.generateCleanupScript()
  }

  private getImportName(importStr: string): string | null {
    // Extract the imported name from various import patterns
    const patterns = [
      /import\s*\{([^}]+)}/,
      /import\s+(\w+)\s+from/,
      /const\s+(\w+)\s*=\s*require/,
      /@import\(['"])([^'"]+)(['"])/
    ]

    for (const pattern of patterns) {
      const match = importStr.match(pattern)
      if (match) {
        const imports = match[1].split(',').map(imp => imp.trim().split(' as ')[0].trim())
        return imports[0] // Return the first imported name
      }
    }

    return null
  }

  private extractDependencyName(importStr: string): string | null {
    // Extract dependency name from import string
    const patterns = [
      /from\s*['"]([^'"]+)['"]/, // import { ... } from 'package'
      /require\(['"])([^'"]+)(['"])/, // require('package')
      /@import\(['"])([^'"]+)['"]/, // @import('package')
      /@Directive\(['"])([^'"]+)['"]/, // @Directive('package')
      /@Injectable\(['"])([^'"]+)['"]/, // @Injectable('package')
    ]

    for (const pattern of patterns) {
      const match = importStr.match(pattern)
      if (match) {
        return match[1]
      }
    }

    return null
  }

  private async generateCleanupScript(): Promise<void> {
    const script = `#!/bin/bash
# Auto-generated cleanup script
# Review before running!

echo "🧹 Code Cleanup Script"
echo "====================="
echo ""
echo "⚠️  WARNING: This script will automatically remove unused code!"
echo "⚠️  Please review the changes before committing!"
echo ""
echo "📋 Would you like to proceed with the cleanup? (y/N)"
read -r response

if [[ ! $response =~ ^[Yy]$ ]]; then
  echo "❌ Cleanup cancelled."
  exit 0
fi

echo ""
echo "🗑️  Starting cleanup..."

# Remove unused dependencies
${this.report.unusedDependencies.map(dep => `npm uninstall ${dep}`).join('\n')}

echo ""
echo "✅ Cleanup complete!"
echo "📝 Please review the changes and run tests before committing!"
`

    fs.writeFileSync('cleanup.sh', script, { mode: 0o755 })
    console.log('\n📝 Generated cleanup.sh script')
  }

  getReport(): CleanupReport {
    return this.report
  }
}

// Main execution
async function main() {
  const analyzer = new CodeCleanupAnalyzer()
  const rootDir = process.argv[2] || '.'

  try {
    const report = await analyzer.analyzeProject(rootDir)

    console.log('\n📊 Summary:')
    console.log(`- Total files analyzed: ${report.totalFiles}`)
    console.log(`- Total lines analyzed: ${report.totalLines}`)
    console.log(`- Files with unused imports: ${report.unusedImports.length}`)
    console.log(`- Files with dead code: ${report.deadCode.length}`)
    console.log(`- Unused dependencies: ${report.unusedDependencies.length}`)

    if (report.unusedImports.length === 0 &&
        report.deadCode.length === 0 &&
        report.unusedDependencies.length === 0) {
      console.log('\n🎉 Great job! No unused code found!')
    }
  } catch (error) {
    console.error('❌ Error analyzing project:', error)
    process.exit(1)
  }
}

// Export for use in other scripts
export { CodeCleanupAnalyzer }

// Run if executed directly
if (require.main === module) {
  main()
}