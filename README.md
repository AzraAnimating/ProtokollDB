# ProtokollDB der FS Medizin

## Build-Instructions
- Um diese Software zu bauen benötigst du die Rust-Toolchain. Die findest du ganz einfach unter https://rustup.rs/
- Wenn du diese installiert hast, kannst du ganz einfach mit einem ``cargo build --release`` das Backend komplett zusammenbauen. 
- Für die derzeitig stabile Version nutz bitte die "main" branch.
- Die fertige Binary ist dann unter target/release/protokolldb zu finden :)
- Viel Spaß!

## Deployment 
- Wenn du eine Binary gebaut hast, musst du nur noch die Config ausfüllen die erstellt wird, sobald du die Binary ausgeführt hast. 
- Unser Deployment sieht in etwa so aus: 
```toml
[database_type.SQLLite]
file_location = "index.db"

[api]
bind_addr = "10.10.10.10"
bind_port = 8080

[authorization.OpenIdConnect]
client_id = "protokolldb"
self_root_url = "http://api.fsmed.cs-rub.de/"
token_url = "https://auth.cs-rub.de/realms/fsmed/protocol/openid-connect/token"
auth_url = "https://auth.cs-rub.de/realms/fsmed/protocol/openid-connect/auth"
revoke_url = "https://auth.cs-rub.de/realms/fsmed/protocol/openid-connect/revoke"
userinfo_url = "https://auth.cs-rub.de/realms/fsmed/protocol/openid-connect/userinfo"

[encryption]
private_key_file = "keys/private"
token_encryption_secret = "ein_unglaublich_sicheres_secret"

[general]
protocol_location = "protocols/"
```
- Am Ende des Tages kann diese Binary überall Laufen, wir empfehlen jedoch einen Dockercontainer zu verwenden.
- Du hast zudem bestimmt bereits die OpenIDConnect Schnittstellen gesehen. Die sind das einzige externe, was vorhanden sein muss um diese API zu betreiben.
- Das ist so, damit der Zugang zu den Protokollen auf Studierende beschränkt werden kann.
