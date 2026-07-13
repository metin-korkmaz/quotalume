# QuotaLume ⚡

> Linux status barında AI kota monitörü — Codex, Claude Code, OpenRouter ve Ollama Cloud token/kredi limitlerini tek bakışta gösterir.

QuotaLume, GNOME (AppIndicator), KDE Plasma ve Sway/Wayland dahil StatusNotifierItem uyumlu tüm Linux masaüstlerinde çalışan minimal, renkli bir sistem tepsisi uygulamasıdır. Ayrı bir arayüzü yoktur — simgeye tıkladığınızda tüm kota bilgileri açılır menüde görüntülenir.

![QuotaLume tray icon concept](https://img.shields.io/badge/platform-Linux-FEDORA?logo=linux&logoColor=white&color=294172)
![Rust](https://img.shields.io/badge/Rust-1.85+-CE422B?logo=rust&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-blue)

## Desteklenen Sağlayıcılar

| Sağlayıcı | Kimlik Doğrulama | Gösterilen Veri |
|-----------|-------------------|-----------------|
| **OpenAI Codex** | OAuth (`~/.codex/auth.json`) | 5 saatlik ve haftalık kullanım penceresi, kredi bakiyesi |
| **Claude Code** | OAuth (`~/.claude/.credentials.json`) | 5 saatlik ve 7 günlük kullanım, model bazlı limitler, ek kullanım |
| **OpenRouter** | API key | Kredi bakiyesi, anahtar limiti, günlük/haftalık kullanım |
| **Ollama Cloud** | API key veya çerez | Oturum (5 saat) ve haftalık kullanım yüzdesi |

## Özellikler

- **Renkli durum simgesi**: 🟢 (yeterli), 🟡 (düşüyor), 🔴 (kritik) ile en düşük kalan kota yüzdesi
- **Detaylı açılır menü**: Her sağlayıcı için kullanım çubukları, plan bilgisi, sıfırlanma zamanı
- **Gizlilik öncelikli**: Mevcut CLI kimlik bilgilerini yeniden kullanır, hiçbir şifre saklamaz
- **Yerel öncelikli**: Tüm veriler makinenizden alınır, bulut hizmeti yoktur
- **Otomatik yenileme**: Varsayılan 5 dakikada bir (yapılandırılabilir)

## Kurulum

### Fedora / RHEL

```bash
# Rust toolchain (henüz kurulu değilse)
sudo dnf install rust cargo

# Derleme ve kurulum
cargo install --path .
```

### Arch Linux

```bash
cargo install --path .
# AppIndicator gerektirir:
sudo pacman -S gnome-shell-extension-appindicator
```

### GNOME için AppIndicator

GNOME kullanıyorsanız tray simgesini görmek için AppIndicator uzantısını etkinleştirin:

```bash
# Fedora
sudo dnf install gnome-shell-extension-appindicator
gnome-extensions enable appindicatorsupport@rgcjonas.gmail.com
```

Kaynaktan veya [extensions.gnome.org](https://extensions.gnome.org/extension/615/appindicator-support/) üzerinden de kurulabilir.

## Yapılandırma

Config dosyası: `~/.config/quotalume/config.toml`

```toml
refresh_seconds = 300  # yenileme aralığı (saniye)

# OpenRouter API key (isteğe bağlı, ortam değişkeni de çalışır)
openrouter_api_key = "sk-or-v1-..."

# Ollama Cloud API key (isteğe bağlı)
ollama_api_key = "..."

# Ollama Cloud çerez (kota pencereleri için)
ollama_cookie_header = "session=..."
```

### Ortam Değişkenleri

| Değişken | Açıklama |
|----------|----------|
| `OPENROUTER_API_KEY` | OpenRouter API anahtarı |
| `OLLAMA_API_KEY` | Ollama Cloud API anahtarı |
| `OLLAMA_COOKIE` | Ollama Cloud oturum çerezi (kota penceresi için) |
| `CODEX_HOME` | Codex config dizini (varsayılan `~/.codex`) |
| `CLAUDE_CONFIG_DIR` | Claude config dizini (varsayılan `~/.claude`) |

## Kullanım

### Normal mod (status bar simgesi)

```bash
quotalume
```

Simge status bar'da görünür. Tıklayarak menüyü açın.

### Tek seferlik mod (terminal çıktısı)

```bash
quotalume --once
```

### Otomatik başlatma

```bash
mkdir -p ~/.config/autostart
cp assets/quotalume.desktop ~/.config/autostart/
```

## Mimari

```
src/
├── main.rs          # Giriş noktası, CLI, tokio runtime
├── lib.rs           # Modül tanımları
├── config.rs        # TOML config, kimlik dosyası yolları
├── credentials.rs   # Codex/Claude OAuth JSON parser'ları
├── model.rs         # Domain modeli (ProviderSnapshot, UsageWindow, vs.)
├── fetch.rs         # HTTP fetch orchestrator (4 sağlayıcı)
├── tray.rs          # ksni StatusNotifierItem tray uygulaması
└── providers/
    ├── codex.rs      # Codex kullanım yanıtı parser'ı
    ├── claude.rs     # Claude OAuth kullanım yanıtı parser'ı
    ├── openrouter.rs # OpenRouter kredi+key API parser'ı
    └── ollama.rs     # Ollama HTML/JSON parser'ı
```

## Geliştirme

```bash
# Testler
cargo test

# Lint
cargo clippy -- -D warnings

# Release derleme
cargo build --release
```

## Teşekkürler

- [CodexBar](https://github.com/steipete/CodexBar) — sağlayıcı veri kaynakları ve API endpoint araştırması için referans
- [ksni](https://github.com/iovxw/ksni) — Rust StatusNotifierItem implementasyonu

## Lisans

MIT — bkz. [LICENSE](LICENSE)