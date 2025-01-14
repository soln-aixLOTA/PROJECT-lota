#!/bin/bash

# Update imports in frontend files
find src/frontend -type f -name "*.tsx" -o -name "*.ts" -o -name "*.jsx" -o -name "*.js" | while read file; do
    # Update component imports
    sed -i 's|from "@/components|from "@/frontend/components|g' "$file"
    # Update hooks imports
    sed -i 's|from "@/hooks|from "@/frontend/hooks|g' "$file"
    # Update utils imports
    sed -i 's|from "@/utils|from "@/frontend/utils|g' "$file"
    # Update app imports
    sed -i 's|from "@/app|from "@/frontend/app|g' "$file"
done

# Update imports in backend files
find src/backend -type f -name "*.ts" -o -name "*.js" | while read file; do
    # Update middleware imports
    sed -i 's|from "../../middleware|from "../middleware|g' "$file"
    # Update common imports
    sed -i 's|from "../../common|from "../common|g' "$file"
done

echo "Import paths updated successfully!" 