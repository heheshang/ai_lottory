#!/bin/bash

echo "🧹 Cleaning up monolithic SuperLotto store..."

# List files that import the old store
FILES_TO_UPDATE=(
  "src/types/index.ts"
  "src/composables/useAlgorithm.ts"
  "src/composables/usePrediction.ts"
  "src/utils/errorHandler.ts"
)

# Create backup of the old store
if [ -f "src/stores/superLotto.ts" ]; then
  cp "src/stores/superLotto.ts" "src/stores/superLotto.ts.backup"
  echo "✅ Backup created: superLotto.ts.backup"
fi

# Update type definitions
if [ -f "src/types/index.ts" ]; then
  echo "🔄 Updating src/types/index.ts..."
  sed -i.bak 's|export \* from '\''\./superLotto'\'|// Removed: superLotto exports|g' "src/types/index.ts"
  echo "✅ Updated type definitions"
fi

# Update composables
for file in "${FILES_TO_UPDATE[@]}"; do
  if [ -f "$file" ]; then
    echo "🔄 Updating $file..."
    # Remove import of useSuperLottoStore
    sed -i.bak '/useSuperLottoStore/d' "$file"
    # Remove any references to the old store
    sed -i.bak '/superLottoStore\./g' "$file"
    sed -i.bak '/from ['\'\'\'][\']superLotto['\'\'\'][\']/g' "$file"
    echo "✅ Updated $file"
  fi
done

# Update error handler
if [ -f "src/utils/errorHandler.ts" ]; then
  echo "🔄 Updating error handler..."
  sed -i.bak '/superLottoError/d' "src/utils/errorHandler.ts"
  sed -i.bak '/types\/superLotto/d' "src/utils/errorHandler.ts"
  echo "✅ Updated error handler"
fi

# Remove old components that depend on the monolithic store
OLD_COMPONENTS=(
  "src/components/super-lotto"
  "src/components/common/BaseCard.vue"
  "src/components/common/EmptyState.vue"
  "src/components/common/LoadingSpinner.vue"
)

echo "🗑️ Checking for old components..."
for component in "${OLD_COMPONENTS[@]}"; do
  if [ -d "$component" ]; then
    echo "⚠️  Directory already removed: $component"
  else
    if [ -f "$component" ] || [ -d "$component" ]; then
      echo "🗑️  Removing old component: $component"
      rm -rf "$component"
    else
      echo "ℹ️  Component not found: $component"
    fi
  fi
done

# Remove the old store file
if [ -f "src/stores/superLotto.ts" ]; then
  echo "🗑️  Removing monolithic store..."
  rm "src/stores/superLotto.ts"
  echo "✅ Removed src/stores/superLotto.ts"
fi

# Clean up backup files
echo "🧹 Cleaning up backup files..."
find . -name "*.bak" -delete

# Check for any remaining references
echo "🔍 Checking for remaining references to old store..."
REMAINING_REFERENCES=$(grep -r "superLotto" src --include="*.vue" --include="*.ts" --include="*.js" | head -5)

if [ -n "$REMAINING_REFERENCES" ]; then
  echo "⚠️  Found remaining references:"
  echo "$REMAINING_REFERENCES"
  echo ""
  echo "🔧 Manual cleanup may be needed for the following files:"
  grep -l "superLotto" src --include="*.vue" --include="*.ts" --include="*.js"
else
  echo "✅ No remaining references to old store found!"
fi

# Update package.json if needed
echo "📦 Checking package.json..."
if grep -q "superLotto" package.json; then
  echo "ℹ️  No direct references in package.json"
else
  echo "✅ Package.json is clean"
fi

echo ""
echo "🎉 Cleanup completed!"
echo ""
echo "📊 Summary:"
echo "   - Monolithic store removed and backed up"
echo "   - Type definitions updated"
echo "   - Composables updated to use new stores"
echo "   - Old components removed"
echo "   - Backup files cleaned up"
echo ""
echo "💡 Next steps:"
echo "   1. Run tests to ensure everything still works"
echo "   2. Update any remaining import statements manually if needed"
echo "   3. Remove any unused dependencies from package.json"
echo "   4. Commit changes"