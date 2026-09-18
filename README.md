# OpenFortiGUI GTK

Frontend independente em Rust para conectar perfis Fortinet usando GTK4, libadwaita e o `openfortivpn` instalado no sistema.

## Dependencias

Em distribuições Debian/Ubuntu, instale as bibliotecas de desenvolvimento antes de compilar:

```bash
sudo apt install build-essential pkg-config libgtk-4-dev libadwaita-1-dev
```

O frontend chama o helper privilegiado via `pkexec` (veja "Autenticação root"
abaixo), então o usuário também precisa de:

```bash
sudo apt install openfortivpn policykit-1
```

No Fedora, os nomes equivalentes são normalmente:

```bash
sudo dnf install gcc pkgconf-pkg-config glib2-devel gtk4-devel libadwaita-devel openfortivpn polkit
```

O arquivo `gobject-2.0.pc` vem do pacote `glib2-devel`. Se o `cargo` reclamar de
um `.pc` ausente, confirme o pacote no Fedora com, por exemplo:

```bash
dnf provides '*/gobject-2.0.pc'
```

## Executar

```bash
cargo run
```

Os perfis ficam em `~/.config/openfortigui-gtk/profiles.json`. A pasta não depende de nenhum arquivo do projeto Qt original e pode ser movida para outro local.

## Instalar em /opt

```bash
cargo build --release
sudo ./install.sh
```

Copia o binário e `assets/icon.svg` para `/opt/openfortigui-gtk/`, cria o
launcher em `/usr/share/applications/org.openfortigui.Gtk.desktop` como link
simbólico para `assets/org.openfortigui.Gtk.desktop` (esse arquivo `.desktop`
precisa existir antes de rodar o script, com `Exec=/opt/openfortigui-gtk/openfortigui-gtk`
e `Icon=/opt/openfortigui-gtk/icon.svg`) e já instala o helper privilegiado e
a ação PolicyKit descritos na seção abaixo.

## Autenticação root (PolicyKit)

O app não pede a senha de root a cada conexão/desconexão: uma ação PolicyKit
dedicada é usada com `auth_admin_keep`, então a senha é solicitada uma vez ao
abrir o app e o PolicyKit mantém essa autorização em cache (renovada por um
keep-alive interno enquanto o app estiver aberto). A senha em si nunca passa
pela memória do app -- fica inteiramente sob custódia do agente de
autenticação do PolicyKit do sistema.

Antes de rodar o app, instale o helper privilegiado e a ação PolicyKit (uma
vez por máquina, requer root) -- já incluído em `sudo ./install.sh`, ou
manualmente:

```bash
sudo install -Dm755 polkit/openfortigui-privileged-helper /usr/lib/openfortigui-gtk/openfortigui-privileged-helper
sudo install -Dm644 polkit/com.openfortigui.vpnhelper.policy /usr/share/polkit-1/actions/com.openfortigui.vpnhelper.policy
```

O helper só aceita `start <config-em-.../openfortigui-gtk/*.conf>`,
`stop <pid-de-um-processo-openfortivpn>` ou `check` (no-op usado para manter a
autenticação viva); ele nunca executa comandos arbitrários vindos do app.

## Funcionalidades

- Cadastro local de perfis VPN.
- Conexão via `pkexec` + helper privilegiado dedicado, com autenticação
  PolicyKit solicitada uma vez no início do app (veja seção acima).
- Login SAML com abertura automática do navegador em `/remote/saml/start`.
- Detecção de certificado de gateway não confiável via sugestão `--trusted-cert` do `openfortivpn`.
- Salvamento automático do certificado confiável no perfil após confirmação do usuário.
