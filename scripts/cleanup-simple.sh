#!/bin/bash

echo "🧹 Starting code cleanup..."

# 1. Backup and remove old monolithic store
if [ -f "src/stores/superLotto.ts" ]; then
  cp "src/stores/superLotto.ts" "src/stores/superLotto.ts.backup"
  rm "src/stores/superLotto.ts"
  echo "✅ Backed up and removed monolithic store"
fi

# 2. Update type definitions
if [ -f "src/types/index.ts" ]; then
  echo "🔄 Updating type definitions..."
  sed -i '' '/export \* from '\''\./superLotto'\'/g' "src/types/index.ts"
  echo "✅ Updated type definitions"
fi

# 3. Update composables that might reference old store
files=(
  "src/composables/useAlgorithm.ts"
  "src/composables/usePrediction.ts"
  "src/utils/errorHandler.ts"
)

for file in "${files[@]}"; do
  if [ -f "$file" ]; then
    sed -i '/useSuperLottoStore/d' "$file"
    sed -i '/superLottoStore/d' "$file"
    sed -i '/from '\''\@\/stores\/superLotto'\''/g' "$file"
    sed -i '/superLottoError/d' "$file"
    echo "✅ Updated $(basename "$file")"
  fi
done

# 4. Remove unused dependencies (check and suggest)
echo "🔍 Checking for unused dependencies..."
if [ -f "package.json" ]; then
  # Common dependencies that might be unused after refactoring
  unused_deps=(
    "echarts"
  "lodash"
    "moment"
    "date-fns"
  )

  for dep in "${unused_deps[@]}"; do
    if grep -q "$dep" package.json; then
      echo "⚠️  Consider removing: $dep"
    fi
  done
fi

# 5. Remove old component files if they exist
old_components=(
  "src/components/super-lotto/AlgorithmSelector.vue"
  "src/components/super-lotto/AnalysisControls.vue"
  "src/components/super-lotto/ColdNumbersChart.vue"
  "src/components/super-lotto/DataTable.vue"
  "src/components/super-lotto/GapPatternsChart.vue"
  "src/components/super-lotto/HotNumbersChart.vue"
)

for component in "${old_components[@]}"; do
  if [ -f "$component" ]; then
    rm "$component"
    echo "✅ Removed old component: $(basename "$component")"
  fi
done

# 6. Update main App.vue if it references old store
if [ -f "src/App.vue" ]; then
  sed -i '/useSuperLottoStore/d' "src/App.vue"
  sed -i '/superLottoStore/d' "src/App.vue"
  echo "✅ Updated App.vue"
fi

echo ""
echo "🎉 Cleanup completed!"
echo ""
echo "📊 Cleanup Summary:"
echo "  - Monolithic store backed up and removed"
echo "  - Type definitions updated"
echo "  - Old imports cleaned from composables"
echo "  - Old components removed"
echo "  - App.vue updated"
echo ""
echo "💡 Next steps:"
echo "  1. Run tests: npm test"
echo "  2. Check for any remaining unused dependencies: npm ls"
echo "  3. Remove unused dependencies: npm uninstall <package>"
echo "  4. Run type check: npm run type-check"
echo "  5. Commit changes: git add . && git commit -m 'Cleanup monolithic store'"