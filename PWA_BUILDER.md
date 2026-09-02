# Instruções para criar APK com PWABuilder

## Pré-requisitos

1. Aplicação deve estar acessível via HTTPS (obrigatório para PWA)
2. Todos os ficheiros gerados estão no sítio correto

## Ficheiros criados

| Ficheiro | Descrição |
|----------|-----------|
| `static/manifest.json` | Manifesto da PWA |
| `static/icons/icon-192.png` | Ícone 192x192 |
| `static/icons/icon-512.png` | Ícone 512x512 |
| `static/sw.js` | Service Worker |

## Passos para criar o APK

### 1. Fazer deploy da aplicação

Opções:
- **Oracle Cloud**: VM com IP público e domínio
- **Railway/Render**: Deploy automático via GitHub
- **VPS própria**: Qualquer servidor com HTTPS

### 2. Configurar HTTPS

Obrigatório para PWA. Use Let's Encrypt (gratuito):

```bash
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d seu-dominio.com
```

### 3. Criar screenshots (opcional mas recomendado)

Coloque em `static/screenshots/`:
- `screenshot-wide.png` (1280x720) - para desktop
- `screenshot-narrow.png` (720x1280) - para mobile

### 4. Testar no PWABuilder

1. Aceda a: https://www.pwabuilder.com/
2. Insira a URL da sua aplicação (ex: https://praisechords.com)
3. Clique em "Start"
4. Aguarde a análise
5. Clique em "Package for stores"
6. Escolha "Android"
7. Clique em "Download"

### 5. Configurações para Android

No PWABuilder, configure:
- **Package ID**: `com.praisechords.app`
- **App name**: PraiseChords
- **Version**: 1.0.0

### 6. Assinar o APK

Para publicar na Play Store, precisa de assinar o APK:

```bash
# Gerar keystore (guarde em local seguro!)
keytool -genkey -v -keystore praisechords.keystore -alias praisechords -keyalg RSA -keysize 2048 -validity 10000

# Assinar o APK
jarsigner -verbose -sigalg SHA1withRSA -digestalg SHA1 -keystore praisechords.keystore app-release-unsigned.apk praisechords

# Otimizar
zipalign -v 4 app-release-unsigned.apk PraiseChords.apk
```

## Estrutura final

```
static/
├── manifest.json      ← Manifesto PWA
├── sw.js              ← Service Worker
├── icons/
│   ├── icon-192.png   ← Ícone 192x192
│   └── icon-512.png   ← Ícone 512x512
└── screenshots/
    ├── screenshot-wide.png
    └── screenshot-narrow.png
```

## Notas importantes

- O service worker só funciona com HTTPS
- Os ícones devem ser PNG
- O manifesto deve estar no caminho `/static/manifest.json`
- Para iOS, o Safari tem suporte limitado a PWA
