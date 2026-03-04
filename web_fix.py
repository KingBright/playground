with open('web/src/pages/mission-control/Dashboard.tsx', 'r') as f:
    content = f.read()

# Remove unused defaultStats
import re
content = re.sub(r'const defaultStats: StatCardData\[\] = \[\s*\{[\s\S]*?\];', '', content)

with open('web/src/pages/mission-control/Dashboard.tsx', 'w') as f:
    f.write(content)

with open('web/src/App.tsx', 'r') as f:
    app_content = f.read()

# Temporarily comment out SystemLogs import if it doesn't exist
app_content = app_content.replace("import { SystemLogs } from './pages/logs/SystemLogs';", "// import { SystemLogs } from './pages/logs/SystemLogs';")
app_content = app_content.replace('<Route path="logs" element={<SystemLogs />} />', '{/* <Route path="logs" element={<SystemLogs />} /> */}')

with open('web/src/App.tsx', 'w') as f:
    f.write(app_content)
