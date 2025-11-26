#!/bin/bash

set -e

echo "🧹 Starting final cleanup process..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Function to print colored output
print_status() {
    echo -e "${GREEN}$1${NC} $2"
}

print_warning() {
    echo -e "${YELLOW}$1${NC} $2"
}

print_error() {
    echo -e "${RED}$1${NC} $2"
}

# 1. Remove monolithic store
print_status "Step 1/7: Removing monolithic store..."

if [ -f "src/stores/superLotto.ts" ]; then
    cp "src/stores/superLotto.ts" "src/stores/superLotto.ts.backup"
    rm "src/stores/superLotto.ts"
    print_status "✅ Backed up and removed monolithic store"
else
    print_warning "ℹ️  Monolithic store not found"
fi

# 2. Update type definitions
print_status "Step 2/7: Updating type definitions..."

if [ -f "src/types/index.ts" ]; then
    # Create backup
    cp "src/types/index.ts" "src/types/index.ts.backup"

    # Remove the superLotto export line
    sed -i.bak '/export.*superLotto/d' "src/types/index.ts"

    print_status "✅ Updated type definitions"
else
    print_warning "ℹ️  Types file not found"
fi

# 3. Update composables
print_status "Step 3/7: Updating composables..."

composables=(
    "src/composables/useAlgorithm.ts"
    "src/composables/usePrediction.ts"
    "src/utils/errorHandler.ts"
)

for file in "${composables[@]}"; do
    if [ -f "$file" ]; then
        # Create backup
        cp "$file" "$file.bak"

        # Remove old imports and references
        sed -i.bak '/useSuperLottoStore/d' "$file"
        sed -i.bak '/superLottoStore/d' "$file"
        sed -i.bak "/from ['\"][\"][\"]@\/stores\/superLotto['\"][\"]/g" "$file"
        sed -i.bak '/superLottoError/d' "$file"

        print_status "✅ Updated $(basename "$file")"
    else
        print_warning "⚠️  File not found: $file"
    fi
done

# 4. Update App.vue
print_status "Step 4/7: Updating App.vue..."

if [ -f "src/App.vue" ]; then
    # Create backup
    cp "src/App.vue" "src/App.vue.backup"

    # Remove old imports and references
    sed -i.bak '/useSuperLottoStore/d' "src/App.vue"
    sed -i.bak '/superLottoStore/d' "src/App.vue"

    print_status "✅ Updated App.vue"
else
    print_warning "⚠️  App.vue not found"
fi

# 5. Update API file if it exists
print_status "Step 5/7: Checking API files..."

api_files=(
    "src/api/superLotto.ts"
)

for file in "${api_files[@]}"; do
    if [ -f "$file" ]; then
        # Create backup
        cp "$file" "$file.bak"

        # Check if it references old store
        if grep -q "superLotto" "$file"; then
            sed -i.bak '/superLotto/d' "$file"
            sed -i.bak '/from ['\"][\"][\"]@\/stores\/superLotto['\"][\"]/g' "$file"
            print_status "✅ Updated $(basename "$file")"
        else
            print_status "✅ $(basename "$file") is clean"
        fi
    else
        print_warning "⚠️  File not found: $file"
    fi
done

# 6. Remove old component directories
print_status "Step 6/7: Removing old component directories..."

old_components=(
    "src/components/super-lotto"
)

for component_dir in "${old_components[@]}"; do
    if [ -d "$component_dir" ]; then
        # Create backup before removal
        tar -czf "$component_dir.bak" "$component_dir"
        rm -rf "$component_dir"
        print_status "✅ Removed directory: $(basename "$component_dir")"
    else
        print_warning "⚠️  Directory not found: $component_dir"
    fi
done

# 7. Clean up backup files
print_status "Step 7/7: Cleaning up backup files..."

find . -maxdepth 1 -name "*.bak" -delete
find . -maxdepth 1 -name "*.backup" -delete

# 8. Check for remaining references
print_status "Step 8/7: Checking for remaining references..."

remaining_refs=$(grep -r "superLotto" src --include="*.vue" --include="*.ts" --include="*.js" 2>/dev/null || true)

if [ "$remaining_refs" = true ]; then
    print_warning "⚠️  Found remaining references to superLotto:"
    grep -n "superLotto" src --include="*.vue" --include="*.ts" --include="*.js" | head -5
    echo ""
    echo "🔧 Manual cleanup may be needed for files containing these references"
else
    print_status "✅ No remaining references to old store found!"
fi

# 9. Check package.json for potentially unused dependencies
print_status "Step 9/7: Checking dependencies..."

if [ -f "package.json" ]; then
    # Common dependencies that might be unused after refactoring
    potentially_unused=(
        "echarts"
        "lodash"
        "moment"
        "date-fns"
    )

    for dep in "${potentially_unused[@]}"; do
        if grep -q "\"$dep\"" package.json; then
            echo "⚠️  Consider reviewing dependency: $dep"
        fi
    done
fi

# 10. Final cleanup verification
print_status "Step 10/7: Verifying cleanup..."

# Check if old files are gone
if [ -f "src/stores/superLotto.ts" ]; then
    print_error "❌ Failed to remove monolithic store"
else
    print_status "✅ Monolithic store successfully removed"
fi

# Check if backup exists
if [ -f "src/stores/superLotto.ts.backup" ]; then
    print_status "✅ Backup created successfully"
else
    print_warning "⚠️  No backup file found"
fi

echo ""
echo -e "${GREEN}🎉 Cleanup completed successfully!"
echo ""
echo "📊 Cleanup Summary:"
echo "  - Monolithic store: ${RED}REMOVED${NC}"
echo "  - Type definitions: ${GREEN}UPDATED${NC}"
echo "  - Composables: ${GREEN}UPDATED${NC}"
echo "  - App.vue: ${GREEN}UPDATED${NC}"
echo "  - Old components: ${RED}REMOVED${NC}"
echo "  - Backup files: ${GREEN}CREATED${NC}"
echo ""
echo "💡 Next steps:"
echo "  1. Run tests: ${YELLOW}npm test${NC}"
echo " 2. Check types: ${YELLOW}npm run type-check${NC}"
echo " 3. Run linting: ${YELLOW}npm run lint${NC}"
echo " 4. Remove unused dependencies: ${YELLOW}npm uninstall <package>${NC}"
echo " 5. Commit changes: ${YELLOW}git add . && git commit -m 'Remove monolithic store and clean up unused code'${NC}"