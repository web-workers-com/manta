# Solución de problemas {#fehlerbehebung}

Soluciones a problemas habituales y dónde registra Manta los errores. Si informas de un error, el archivo de registro ayuda mucho.

## El archivo de registro {#protokoll}

Manta escribe los errores y los eventos importantes en:

`~/Library/Logs/Manta/manta.log`

Para encontrarlo:

1. Elige **Ayuda** → **Mostrar archivo de registro en Finder** en la barra de menús, o abre la paleta de comandos con **⇧⌘P** y elige allí **Mostrar archivo de registro en Finder**.
2. Finder abre la carpeta de registros (`~/Library/Logs/Manta`) con el archivo `manta.log`, en el propio Finder, aunque Manta sea tu gestor de archivos predeterminado.

Cada mensaje de error que Manta te muestra está ahí con el error técnico, la ruta y la ubicación (local, servidor…). El archivo rota al llegar a 5 MB; se conserva una copia anterior.

> [!TIP]
> Para obtener registros más detallados, abre Manta desde Terminal con `MANTA_LOG=debug /Applications/Manta.app/Contents/MacOS/manta`.

> [!NOTE]
> El archivo de registro puede contener nombres de archivos y de servidores. Revísalo antes de compartirlo. Las contraseñas, frases de contraseña y claves secretas no aparecen en él, tampoco para SFTP, FTP y S3.

Los plugins tienen sus propios registros: **Ajustes** → **Plugins** → **Mostrar registro** en la tarjeta del plugin.

## Informar de un error {#fehler-melden}

¿Has encontrado un error? Infórmanos en GitHub. Lo más fácil es **Ayuda** → **Informar de un problema…** (también en la paleta de comandos, escribiendo «error» o «bug», y en **Ajustes** → **General** → **Acerca de Manta** → **Comentarios**). Manta abre el formulario de informe de errores en tu navegador con la versión de Manta, la versión de macOS y el chip (Apple silicon o Intel) ya rellenados. Manta no envía nada por sí mismo: ves el formulario y decides qué envías. Necesitas una cuenta gratuita de GitHub.

Lo que ayuda a encontrar el error:

- **Versión de Manta y de macOS**: las rellena Manta; si no, en **Manta** → **Acerca de Manta** y en el menú Apple → «Acerca de este Mac».
- **Pasos para reproducirlo**: ¿qué hiciste, paso a paso?
- **Comportamiento esperado y real**: ¿qué debería haber pasado y qué pasó en su lugar? El mensaje de error exacto ayuda.
- **Una captura de pantalla** (⇧⌘4) o una grabación de pantalla corta (⇧⌘5).
- **Las líneas relevantes del archivo de registro** `~/Library/Logs/Manta/manta.log`: ábrelo con **Ayuda** → **Mostrar archivo de registro en Finder**, consulta [El archivo de registro](#protokoll).

> [!IMPORTANT]
> Los informes de errores en GitHub son públicos. Antes de pegar un registro o una captura, elimina o sustituye los datos privados: nombres de usuario y rutas como `/Users/anna/…`, nombres de servidores y hosts, direcciones IP, nombres de buckets.

> [!CAUTION]
> Nunca informes de vulnerabilidades de seguridad en un issue público. Infórmanos de forma privada mediante [Report a vulnerability](https://github.com/web-workers-com/manta/security/advisories/new) en GitHub, como se describe en la [política de seguridad](https://github.com/web-workers-com/manta/blob/main/SECURITY.md).

¿Tienes una idea o una sugerencia? **Ayuda** → **Sugerir una idea…** abre el formulario de ideas. Todas las opciones están también en [github.com/web-workers-com/manta/issues/new/choose](https://github.com/web-workers-com/manta/issues/new/choose).

## Si Manta se cierra inesperadamente {#absturz}

Si Manta falla, guarda un informe de fallo en `~/Library/Logs/Manta`: un archivo cuyo nombre empieza por `crash-` seguido de la fecha y la hora (UTC). El informe contiene solo lo necesario para encontrar el error: la versión de Manta y de macOS, el chip, el mensaje de error y las direcciones del programa en el momento del fallo (el «backtrace»). Tu carpeta de inicio aparece como `~`, y los nombres de usuario, servidor y host se sustituyen por marcadores como `<user>` y `<redacted>`.

La próxima vez que abras Manta, aparece una vez un aviso en la esquina inferior derecha: **Manta se cerró inesperadamente la última vez**.

- **Informar en GitHub** abre el formulario de informe de errores con las versiones y un breve resumen del informe ya rellenados. Adjunta el archivo del informe al formulario (arrástralo al campo de texto): un enlace no puede llevar un archivo. No se envía nada hasta que tú mismo envíes el formulario en GitHub.
- **Mostrar informe** abre el informe en tu editor de texto, para que veas exactamente qué contiene.
- **Ignorar** cierra el aviso. El informe se queda en la carpeta; Manta conserva los 10 más recientes.

> [!NOTE]
> Manta nunca envía informes de fallo por sí mismo y no contiene ningún seguimiento. Si no quieres informar de un fallo, ignora el aviso o elimina los archivos `crash-…` de `~/Library/Logs/Manta`.

## Manta no se abre (Gatekeeper) {#gatekeeper}

**Problema:** En el primer arranque, macOS indica *No se ha abierto Manta* (*Apple no ha podido verificar que Manta no contenga software malicioso…*) o, en versiones anteriores de macOS, *No se puede abrir “Manta” porque Apple no puede comprobar si contiene software malicioso*.

**Causa:** Manta no está notarizada por Apple. macOS bloquea estas apps en el primer arranque.

**Solución:**

1. Haz clic en **Aceptar** en el mensaje (no en **Trasladar a la Papelera**).
2. Abre **Ajustes del Sistema** → **Privacidad y seguridad** y desplázate hasta la sección **Seguridad**.
3. Haz clic en **Abrir igualmente** junto a *Se ha bloqueado Manta…*, otra vez en **Abrir igualmente** cuando macOS pregunte, y confirma con tu contraseña o con Touch ID.

![Ajustes del Sistema, Privacidad y seguridad: el botón Abrir igualmente junto al aviso sobre Manta](images/gatekeeper-open-anyway.png)

Si no aparece **Abrir igualmente**, abre Manta otra vez y mira justo después: el botón solo se mantiene una hora aproximadamente. Todos los pasos con imágenes, y la vía por Terminal, están en [El primer arranque: Gatekeeper](01-erste-schritte.md#gatekeeper).

Antes de macOS 15: haz clic derecho (o Control + clic) en Manta en la carpeta **Aplicaciones** → **Abrir** → **Abrir**.

## “macOS ha bloqueado el acceso” {#berechtigungen}

**Problema:** Una carpeta no se abre, o Manta indica *macOS ha bloqueado el acceso a “…” (operación no permitida)*.

**Causa:** No has permitido que Manta acceda a esta carpeta (o has hecho clic en “No permitir” por error), o la carpeta necesita Acceso total al disco.

**Solución:**

1. Abre **Ajustes del Sistema** → **Privacidad y seguridad** → **Archivos y carpetas**.
2. Busca **Manta** y activa la carpeta (Escritorio, Documentos, Descargas, volúmenes extraíbles, volúmenes de red).
3. Para carpetas como `~/Library/Mail`: **Acceso total al disco** → activa **Manta**.
4. Cierra Manta y vuelve a abrirlo.

Si Manta no aparece en la lista, macOS todavía no ha preguntado. Abre la carpeta una vez en Manta y aparecerá la pregunta.

“No tienes permiso para abrir esta carpeta” es otra cosa: faltan los permisos Unix normales. Pide acceso al propietario de la carpeta o cambia los permisos en Finder (**Obtener información**).

## Funciones que controlan Finder o Terminal {#automation}

**Problema:** “Mostrar en Finder”, “Abrir en Terminal.app” o trasladar duplicados a la papelera no funciona.

**Solución:** **Ajustes del Sistema** → **Privacidad y seguridad** → **Automatización** → en **Manta**, activa **Finder** y **Terminal**.

## FTP: las carpetas no cargan, errores de “TLS” {#ftp-datenkanal}

**Problema:** Iniciar sesión en un servidor FTPS funciona, pero las carpetas se quedan vacías o no terminan de cargar, o Manta indica “la red bloquea las conexiones de datos FTP cifradas”.

**Causa:** Algunas redes de empresa y cortafuegos interrumpen las conexiones de datos FTP cifradas.

**Solución:**

1. Haz clic derecho en el favorito → **Editar conexión…** (o **⌘K** para una conexión nueva).
2. Ajusta **Canal de datos** a **Automático**.
3. Vuelve a conectarte.

Manta envía entonces los listados y los archivos sin cifrar; el inicio de sesión sigue cifrado. Un escudo de advertencia en la etiqueta de ubicación lo indica.

> [!WARNING]
> No es una buena solución para datos confidenciales. En ese caso, usa SFTP, otra red o una VPN, y ajusta el canal de datos a **Obligatorio**.

Más indicaciones sobre FTP:

- “No se ha podido establecer una conexión segura”: el servidor no admite FTPS en este puerto (prueba **FTP** si es aceptable en tu red) o su certificado no es de confianza.
- Manta no admite FTPS implícito (puerto 990).

## Un contenedor sin shell {#container-ohne-shell}

**Problema:** Un contenedor se abre en modo de solo lectura, o Manta indica “El contenedor no tiene shell”.

**Causa:** Manta necesita `sh` en el contenedor para listar y modificar archivos. Las imágenes distroless y scratch no tienen shell.

**Solución:**

- Leer y copiar hacia fuera (**F5**, arrastrar) sigue funcionando, mediante `docker cp`.
- Para hacer cambios necesitas una imagen con shell (una variante de depuración de la imagen, por ejemplo).
- Los pods de Kubernetes necesitan `sh` incluso para leer.
- Los contenedores detenidos también son de solo lectura: inícialos para hacer cambios.

## Docker o Kubernetes no aparecen {#container-fehlen}

- **“Docker no se está ejecutando”**: abre Docker Desktop.
- **“Docker no está instalado”**: instala Docker Desktop o asegúrate de que el comando `docker` está en tu PATH. Manta también busca en `/opt/homebrew/bin`, `/usr/local/bin` y dentro de la app Docker Desktop.
- **“kubectl no está instalado”**: `brew install kubectl`.
- **“No se puede acceder al clúster de Kubernetes”**: comprueba el contexto y tu VPN, o inicia tu clúster local.

## Conexiones a servidores {#server}

| Mensaje | Solución |
|---|---|
| “La clave del servidor ha cambiado” | La clave no coincide con `~/.ssh/known_hosts`; Manta no se conecta (posible ataque de intermediario). Pide la huella al administrador. Si de verdad se ha reinstalado el servidor, elimina la entrada antigua con el comando del diálogo (`ssh-keygen -R host`). |
| “Servidor desconocido” | Primera conexión con este servidor. Compara la huella y haz clic en **Conectar y recordar**; consulta [Conexiones](08-verbindungen.md). |
| “No se ha encontrado la contraseña guardada en el Llavero” | Se ha eliminado el ítem, por ejemplo en Acceso a Llaveros. Introduce la contraseña y deja activado **Recordar contraseña**. |
| “No hay ninguna clave utilizable en ssh-agent” | `ssh-add ~/.ssh/id_ed25519` |
| “El reloj de tu Mac no está en hora” | **Ajustes del Sistema** → **General** → **Fecha y hora** → ajustar automáticamente. |
| “El bucket está en otra región” | Ajusta la región del diálogo a la indicada. |
| “Se ha perdido la conexión con el servidor” | Comprueba la red y después haz clic en **Reintentar**. Manta vuelve a iniciar sesión en el servidor por sí solo. |

## Una transferencia no avanza {#uebertragungen}

| Lo que ves | Solución |
|---|---|
| “Esperando conexión… (intento 5, siguiente en 16 s)” | La red o el servidor no están disponibles. Manta sigue intentándolo y continúa en el mismo punto. Comprueba la Wi-Fi o la VPN; **Reintentar ahora** inicia el siguiente intento al instante. |
| **No hay conexión a internet** en la parte superior de **Transferencias** | Tu Mac no tiene conexión. En cuanto vuelva a tenerla, las transferencias continuarán solas. |
| “Requiere una conexión con…” | Tras un reinicio falta la conexión con el servidor. Haz clic en **Reconectar** e introduce la contraseña. |
| “En pausa · …” | Tú (o **Pausar todo**) has pausado la tarea. Haz clic en **Reanudar**. |
| Archivos ocultos `.NOMBRE.manta-part` en el destino | Archivos parciales de una tarea interrumpida. Déjalos mientras quieras reanudar la tarea; si no, puedes eliminarlos. |

Consulta [Pausar y reanudar transferencias](05-dateien.md#pausieren).

## Una actualización falla {#updates}

Consulta [Actualizaciones](17-updates.md#blockiert). Normalmente, Manta no se está ejecutando desde la carpeta **Aplicaciones**.

## Nube: “Eliminar descarga” no funciona {#cloud}

Con Google Drive, OneDrive, Dropbox y otros, decide la app del proveedor. Usa Finder: clic derecho → **Eliminar descarga**. Consulta [Almacenamiento en la nube](09-cloud.md#download-entfernen).

## Una carpeta muestra contenido desactualizado {#aktualisieren}

Pulsa **⌘R**. Manta solo ve los cambios hechos por otras apps o por el servidor después de actualizar.

## Restablecer Manta {#zuruecksetzen}

Como último recurso:

1. Cierra Manta.
2. Renombra `~/Library/Application Support/Manta` (por ejemplo, a `Manta antiguo`).
3. Abre Manta. Se iniciará como si fuera la primera vez.

Tus favoritos y tus acciones rápidas siguen en la carpeta renombrada.
