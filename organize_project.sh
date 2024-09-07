#!/bin/bash

# Ensure the script is run from the project root
if [ ! -f "Dockerfile" ]; then
  echo "Please run this script from the project root directory."
  exit 1
fi

# Move React app files to the src directory
echo "Moving React app files to the src directory..."
mkdir -p src/components
mv rapidmq-dashboard/src/App.css src/ 2>/dev/null
mv rapidmq-dashboard/src/App.js src/ 2>/dev/null
mv rapidmq-dashboard/src/App.test.js src/ 2>/dev/null
mv rapidmq-dashboard/src/index.css src/ 2>/dev/null
mv rapidmq-dashboard/src/index.js src/ 2>/dev/null
mv rapidmq-dashboard/src/logo.svg src/ 2>/dev/null
mv rapidmq-dashboard/src/reportWebVitals.js src/ 2>/dev/null
mv rapidmq-dashboard/src/setupTests.js src/ 2>/dev/null
mv rapidmq-dashboard/src/components/ClusterStatus.js src/components/ 2>/dev/null
mv rapidmq-dashboard/src/components/MessageMetrics.js src/components/ 2>/dev/null
mv rapidmq-dashboard/src/components/SystemHealth.js src/components/ 2>/dev/null
mv rapidmq-dashboard/src/components/VoiceCommands.js src/components/ 2>/dev/null

# Move public files to the public directory
echo "Moving public files to the public directory..."
mv rapidmq-dashboard/public/* public/ 2>/dev/null

# Move package.json and package-lock.json to the root directory
echo "Moving package.json and package-lock.json to the root directory..."
mv rapidmq-dashboard/package.json . 2>/dev/null
mv rapidmq-dashboard/package-lock.json . 2>/dev/null

# Remove the empty rapidmq-dashboard directory
echo "Removing the empty rapidmq-dashboard directory..."
rm -rf rapidmq-dashboard

echo "Project files have been organized successfully."