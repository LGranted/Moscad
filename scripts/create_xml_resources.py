import os

os.makedirs('src-tauri/gen/android/app/src/main/res/xml', exist_ok=True)

xml_content = '''<?xml version="1.0" encoding="utf-8"?>
<network-security-config>
    <base-config cleartextTrafficPermitted="false">
        <trust-anchors>
            <certificates src="system"/>
        </trust-anchors>
    </base-config>
</network-security-config>'''

with open('src-tauri/gen/android/app/src/main/res/xml/network_security_config.xml', 'w') as f:
    f.write(xml_content)
print('network_security_config.xml created')
