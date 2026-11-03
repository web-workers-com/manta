# Terminal {#terminal}

Debajo de cada panel puedes mostrar un terminal que sigue a la carpeta del panel, y viceversa. Se ejecuta donde está el panel: en tu Mac, en un servidor SFTP o en un contenedor.

![Un panel con un terminal debajo](images/terminal.png)

## Mostrar el terminal {#einblenden}

- Pulsa **⌃`** (**Mostrar/ocultar terminal**).
- O elige **Mostrar/ocultar terminal** en la paleta de comandos.

El terminal aparece debajo del panel activo. Arrastra su borde superior para cambiar su altura.

Más comandos en la paleta de comandos:

- **Nueva sesión de terminal**: inicia una shell nueva.
- **Borrar terminal**: limpia la pantalla.

Si cierras la shell (con `exit`, por ejemplo), el terminal muestra “[Proceso finalizado: pulsa Retorno para reiniciar]”. Si el panel muestra un servidor o un recurso compartido (SFTP, SMB, Docker, Kubernetes), `exit` también lo abandona allí: el panel vuelve a la carpeta de tu Mac que mostraba antes y la conexión se cierra, salvo que otra pestaña aún la muestre. Si la shell se interrumpe (conexión perdida, contenedor detenido), el panel se queda donde está.

## El panel y el terminal se siguen mutuamente {#folgen}

**Del panel al terminal:** cuando cambias de carpeta en el panel, la shell hace `cd` a ella en cuanto está inactiva, es decir, cuando no se está ejecutando ningún comando. Hasta entonces, una nota indica *Cambiará a “carpeta” cuando la shell esté lista*. Lo que hayas escrito a medias se conserva.

**Del terminal al panel:** escribe `cd ~/Projects` en el terminal y el panel de arriba mostrará esa carpeta. Desactívalo con **El panel sigue a cd del terminal** (consulta [Ajustes](#einstellungen)).

Tu prompt y la configuración de tu shell no cambian.

## Terminal en contenedores y en servidores {#container-ssh}

El terminal se ejecuta donde está el panel:

| El panel muestra… | Terminal |
|---|---|
| una carpeta local (también carpetas en la nube) | tu shell normal en el Mac |
| un contenedor de Docker | una shell dentro del contenedor (`docker exec`) |
| un pod de Kubernetes | una shell dentro del pod (`kubectl exec`) |
| un servidor SFTP | `ssh` al servidor, con tu configuración SSH y tu ssh-agent |
| S3, FTP, un archivo comprimido | ningún terminal (“No hay terminal para esta ubicación”) |

- Manta no instala nada en el contenedor ni en el servidor para esto.
- En las conexiones SFTP con contraseña, escribes la contraseña en el terminal. Un archivo de clave que no esté en tu configuración SSH ni en tu agente también se pide en el terminal.
- Cada panel conserva hasta cuatro shells (local, contenedores, servidores). Cuando vuelves, la shell anterior sigue ahí con su historial.
- Si un contenedor está detenido, el terminal indica *El contenedor “nombre” está detenido*. Inícialo y haz clic en la nota.

Clic derecho → **Abrir shell en el contenedor** inicia una shell en la carpeta del contenedor que estás viendo.

### `ssh` en el terminal local {#ssh-erkennen}

Si escribes `ssh` tú mismo en el terminal local (p. ej., `ssh anna@servidor` o un host de tu `~/.ssh/config`), Manta lo detecta. Mientras la conexión está activa, aparece **Abrir anna@servidor en el panel** arriba a la derecha del terminal. Al hacer clic, el panel muestra el servidor por SFTP: Manta reutiliza una conexión abierta con él, si no un favorito que coincida y, si no, abre el diálogo de conexión ya rellenado (solo introduces la contraseña). A partir de ahí, el panel y el terminal se siguen como con un contenedor, y puedes copiar y trasladar entre los paneles.

- El terminal inicia entonces su propia shell en el servidor; con inicio de sesión por contraseña, la vuelve a pedir allí. La sesión `ssh` que escribiste sigue en segundo plano hasta que vuelvas a la carpeta local.
- `ssh` se detecta en zsh (la shell predeterminada de macOS), también mediante un alias. `ssh` con un comando detrás (`ssh servidor uptime`) o solo para un túnel (`-N`) no ofrece nada.

## Acciones de archivo en el terminal {#dateiaktionen}

Cuando copias, mueves, eliminas o renombras algo en Manta, el terminal muestra el comando de shell equivalente (como `cp -R …`, `scp …`, `docker cp …` o `aws s3 cp …`) seguido del resultado (“listo”, “error: …”). Trasladar a la papelera aparece como `mv … ~/.Trash/` y una eliminación definitiva como `rm -r …`.

> [!NOTE]
> Es solo una visualización. Manta hace el trabajo por sí mismo; no se ejecuta nada en la shell. Aun así, puedes copiar los comandos, por ejemplo para tus propios scripts.

En las shells remotas (contenedores, SSH), Manta muestra estos comandos como una tarjeta sobre el terminal en lugar de escribirlos en la shell.

Para desactivarlo: **Mostrar acciones de archivo en el terminal**, en los ajustes.

## Ajustes {#einstellungen}

Abre **Ajustes** (**⌘,**) → **Plugins** → **Terminal** → **Ajustes**.

| Ajuste | Significado |
|---|---|
| **Shell** | Qué shell se inicia (`/bin/zsh`, por ejemplo). |
| **Tamaño de letra** | El tamaño de la letra del terminal. |
| **Historial** | Cuántas líneas puedes desplazarte hacia atrás. |
| **Usar Opción como tecla Meta** | Opción + tecla envía ESC + tecla (para atajos al estilo de Emacs) en lugar de escribir caracteres especiales. |
| **El panel sigue a cd del terminal** | Un `cd` en el terminal cambia la carpeta del panel. |
| **Mostrar acciones de archivo en el terminal** | Muestra las acciones de archivo como comandos de shell (consulta más arriba). |

> [!TIP]
> En muchos teclados no estadounidenses (también en el español) necesitas Opción para `@`, `{`, `[` y `|`. Deja **Usar Opción como tecla Meta** desactivado salvo que uses atajos al estilo de Emacs.

## El teclado en el terminal {#tastatur}

Mientras el terminal tiene el foco, estos atajos pertenecen al terminal en lugar de a Manta: **⌘C**, **⌘V**, **⌘A**, **⌘K**, **⌘←**, **⌘→**, **⌘⌫** y **⌘Z**. Otros atajos de Manta, como **⇧⌘P**, siguen funcionando. Haz clic en la lista de archivos para devolver el teclado a Manta.

## Limitaciones {#grenzen}

- `sh` de busybox (Alpine) y bash 3 no pueden conservar lo escrito a medias cuando el terminal sigue al panel (bash 4 o posterior y zsh sí pueden).
- En contenedores con una shell distinta de bash o de `sh` POSIX, el terminal solo sigue en un sentido.
- No hay terminal para S3, FTP ni archivos comprimidos.
