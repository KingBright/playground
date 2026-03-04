with open('web/src/App.tsx', 'r') as f:
    app_content = f.read()

import re
app_content = re.sub(r'<Route path="logs" element=\{<SystemLogs />\} />', '', app_content)

with open('web/src/App.tsx', 'w') as f:
    f.write(app_content)
