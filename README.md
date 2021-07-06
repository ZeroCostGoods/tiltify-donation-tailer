# tiltify-donation-tailer
Application for tailing Tiltify donations into a log

## Config

A config file must be placed in the directory where the exe is located with the name `tiltify-donation-tailer.json`.

```json
{
    "access_token": "your-access-token-from-tiltify",
    "campaign_id": "12345"
}
```

## Building Release exe

```powershell
$env:RUSTFLAGS="-C link-arg=-s"; cargo build --release
```