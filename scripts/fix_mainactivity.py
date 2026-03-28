import os
os.makedirs('src-tauri/gen/android/app/src/main/java/com/lbjlaq/antigravity_tools', exist_ok=True)
content = 'package com.lbjlaq.antigravity_tools\n\nimport app.tauri.TauriActivity\n\nclass MainActivity : TauriActivity()\n'
with open('src-tauri/gen/android/app/src/main/java/com/lbjlaq/antigravity_tools/MainActivity.kt', 'w') as f:
    f.write(content)
print('MainActivity.kt written')
