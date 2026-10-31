# Ajustes {#einstellungen}

Todos los ajustes de Manta, sección por sección. Todos los cambios se aplican al instante; nunca hace falta reiniciar.

## Abrir los ajustes {#oeffnen}

- **⌘,**
- **Manta** → **Ajustes…** en la barra de menús
- el engranaje de la barra de herramientas
- **Ajustes…** en la paleta de comandos

Los ajustes son una ventana propia con las secciones **General** y **Plugins**. Ciérrala con el botón rojo de la ventana o con **⌘W**. **⌘,** la vuelve a traer al frente.

Mientras los ajustes están abiertos, la ventana principal espera: se oscurece ligeramente y los clics y atajos en ella no hacen nada; un clic trae los ajustes al frente con el sonido de alerta. Cierra los ajustes para continuar. La ventana de ayuda, en cambio, puede quedarse abierta mientras trabajas.

![Ajustes → General](images/settings-general.png)

## General {#allgemein}

### Apariencia {#erscheinungsbild}

| Ajuste | Opciones |
|---|---|
| **Apariencia** | **Automático** (sigue el modo claro u oscuro del sistema), **Claro**, **Oscuro**. También en la paleta de comandos: “Apariencia: Claro/Oscuro/Automático (sistema)”. |
| **Color de acentuación** | **Azul**, **Morado**, **Rosa**, **Rojo**, **Naranja**, **Amarillo**, **Verde**, **Grafito**: para la selección, los botones y el foco. Todos los colores se leen bien en modo claro y oscuro. |

### Idioma {#sprache}

**Idioma**: **Sistema** (por omisión) o uno de los idiomas de la app, cada uno con su propio nombre (**Deutsch**, **English**, **简体中文**, **Español**, **Français**, **Italiano**, **Русский**). El cambio se aplica al instante. “Sistema” sigue tus idiomas preferidos de macOS (o inglés si Manta no habla ninguno de ellos). Las fechas (“Hoy 14:02”), los tamaños (“1,2 MB”) y los números siguen el idioma. Los plugins integrados cambian con él; los nombres de archivo, la salida de la shell y tus propias acciones rápidas no cambian.

### Gestor de archivos predeterminado {#standard-dateimanager}

**Usar Manta como gestor de archivos predeterminado**: cuando está activado, “Mostrar en Finder” y las carpetas de otras apps se abren en Manta. Debajo ves qué app tiene ahora esa función (“Actualmente: …”) y qué app la recupera cuando lo desactivas. Lo que permite macOS se explica en [Primeros pasos](01-erste-schritte.md#standard-dateimanager).

El mismo interruptor está en el menú **Manta** (**Manta como gestor de archivos predeterminado**, con una marca cuando está activado) y en la paleta de comandos (**Usar Manta como gestor de archivos predeterminado…** o **Devolver el gestor de archivos predeterminado a…** cuando Manta tiene esa función). Al activarlo desde ahí, se hace la misma pregunta que en el primer arranque, con **Cancelar**; al desactivarlo, la función se devuelve al instante.

Solo está disponible cuando Manta se ejecuta desde la carpeta **Aplicaciones** (“Primero traslada Manta a la carpeta Aplicaciones.”).

### Archivos {#dateien}

| Ajuste | Significado |
|---|---|
| **Mostrar archivos ocultos** | Para pestañas nuevas y al abrir la app. **⇧⌘.** los muestra u oculta en la pestaña actual. |
| **Abrir archivos comprimidos como carpetas** | Return o un doble clic explora los archivos ZIP y TAR en el panel (solo lectura). **⌥↩** los abre con la app predeterminada. Consulta [Archivos comprimidos](10-archive.md). |
| **Las pestañas nuevas se abren en** | Dónde se abren **⌘T**, **Nueva pestaña** en el menú del Dock y un inicio sin sesión guardada: **La misma carpeta** (por omisión; al iniciar, tu carpeta de inicio), **Inicio**, **Escritorio**, **Documentos**, **Descargas** u **Otra…** (una carpeta que elijas). Como «Las ventanas nuevas del Finder muestran» en el Finder. |
| **Recordar la visualización por carpeta** | Cada carpeta se abre en la visualización en la que se mostró por última vez (Iconos, Lista, Columnas, Galería). Desactivado (por omisión): la visualización pertenece al panel. Consulta [Visualizaciones](03-ansichten.md#pro-ordner). |
| **Botones laterales del ratón** | Los botones del pulgar retroceden y avanzan en el panel bajo el puntero, como **⌘[** y **⌘]**, también en la Ayuda. **Atrás / Adelante** (predeterminado), **Adelante / Atrás (invertido)** o **Desactivado**. |
| **Cargar miniaturas de archivos del servidor** | Descarga imágenes pequeñas (menos de 5 MB) en segundo plano. Desactivado (por omisión): los archivos del servidor muestran el icono de su tipo. Consulta [Miniaturas](03-ansichten.md#server-vorschaubilder). |
| **Iconos de archivo de macOS** | Los archivos muestran el icono que macOS tiene para su tipo y las apps el suyo propio, como en el Finder, también en servidores. Desactivado: los símbolos de Manta. Predeterminado: activado. |

### Vista previa {#vorschau}

**La barra espaciadora abre**: **Vista Rápida (macOS)** o **Vista previa integrada**. Los archivos de servidores y de archivos comprimidos siempre usan la vista previa integrada. Consulta [Vista Rápida y vista previa](06-schnellansicht.md).

### Copiar y mover {#kopieren-bewegen}

**Al soltar archivos**: **Preguntar (Copiar preseleccionado)** (por omisión), **Copiar siempre** o **Mover siempre**. Se aplica al arrastrar entre paneles y desde otras apps. “Preguntar” muestra el diálogo con Copiar / Mover; las dos opciones “siempre” empiezan al instante. **⌘** al soltar mueve y **⌥** copia, sea cual sea el ajuste. No afecta a lo que se suelta en la Papelera ni en Favoritos. Consulta [Arrastrar y soltar](05-dateien.md#drag-drop).

**Si un ítem ya existe**: **Preguntar** (por omisión), **Omitir**, **Reemplazar si es más reciente**, **Reemplazar** o **Conservar ambos**. Esta opción está preseleccionada al copiar o mover (también al soltar); puedes cambiarla cada vez. Cuando una tarea empieza sin diálogo al soltar (“Copiar siempre” / “Mover siempre”), se aplica este ajuste. Con **Preguntar**, una tarea se detiene en cada ítem que ya existe y te pregunta. Las instalaciones nuevas preguntan; si ya usabas Manta, se conserva tu ajuste guardado (incluida la antigua opción por omisión **Omitir**); cámbialo aquí. Consulta [Conflictos de nombres](05-dateien.md#konflikte).

### Nube {#cloud}

**Google Drive**: muestra si Google Drive para ordenadores está configurado y, si no, el enlace de descarga. **Aviso en la barra lateral** activa o desactiva la fila “Conectar Google Drive…”. **Experto: tu propio cliente OAuth de Google** es solo para usuarios avanzados; consulta [Google Drive para expertos](09-cloud.md#experte).

### Actualizaciones {#updates}

| Ajuste | Significado |
|---|---|
| **Buscar actualizaciones al abrir** | Discretamente, en segundo plano. No se instala nada sin preguntarte. Por omisión: activado. |
| **Canal** | **Estable** o **Beta**. Beta ofrece funciones nuevas antes, pero puede contener errores. |
| **Buscar ahora** | Busca al instante. Al lado: “Última búsqueda: …”. |
| **Omitida: Manta …** / **Restablecer** | Aparece cuando has omitido una versión. Esa versión y las anteriores no se ofrecen hasta que haces clic en **Restablecer**. |

Si a tu versión le falta la clave de firma de actualizaciones, esta sección indica “Las actualizaciones no están configuradas en esta versión”. Consulta [Actualizaciones](17-updates.md).

### Acerca de Manta {#ueber}

Manta es gratuito y de código abierto. **Sitio web y soporte** → **manta-filemanager.com** abre [el sitio web](https://manta-filemanager.com/) con novedades, ayuda y contacto. **Licencias de terceros** → **Mostrar** abre los avisos de licencia de los componentes de código abierto en los que se basa Manta. **Código fuente** → **GitHub** abre [el repositorio](https://github.com/web-workers-com/manta); las contribuciones son bienvenidas. **Licencia MIT** → **Leer** muestra la licencia: puedes usar, modificar y compartir Manta libremente. El sitio web, el código fuente y las licencias de terceros están también en el menú **Ayuda**. **Comentarios** → **Informar de un problema** abre el formulario de informe de errores de GitHub con las versiones de Manta y macOS rellenadas; consulta [Informar de un error](19-fehlerbehebung.md#fehler-melden).

## Plugins {#plugins}

Arriba: **Instalar plugin…**, **Abrir carpeta de plugins** y **Recargar**. Debajo hay tres secciones:

- **Integrados**: los plugins integrados (Git, Terminal, Tamaños de carpeta, Renombrar en lote, Vista previa de Markdown, Acciones rápidas, Info del proyecto, Herramientas de imagen, Duplicados, Carga del sistema).
- **Instalados**: los plugins externos que has instalado.
- **Disponibles**: plugins incluidos con Manta, listos para instalar.

Cada tarjeta muestra el nombre, la descripción, el estado (**En ejecución**, **Iniciando…**, **Desactivado**, **Error**) y los botones **Activar**/**Desactivar**, **Ajustes** (si los hay) y **Mostrar registro**. Más información en [Tus propios plugins](15-eigene-plugins.md).

### Ajustes de los plugins {#plugin-einstellungen}

| Plugin | Ajustes |
|---|---|
| **Terminal** | Shell, Tamaño de letra, Historial, Usar Opción como tecla Meta, El panel sigue a cd del terminal, Mostrar acciones de archivo en el terminal; consulta [Terminal](11-terminal.md#einstellungen). |
| **Tamaños de carpeta** | **Calcular automáticamente el tamaño de las carpetas** o **Solo cuando se solicite**; consulta [Tamaños de carpeta](14-plugins.md#ordnergroessen). |
| **Vista previa de Markdown** | **Cargar imágenes de internet (http/https)**: desactivado por omisión. |
| **Duplicados** | **Omitir paquetes (.app, .photoslibrary, …)**, **Incluir archivos ocultos**, **Editar settings.json…** para exclusiones y un comando de papelera personalizado. |
| **Carga del sistema** | **Mostrar la carga de la GPU**; consulta [Carga del sistema](14-plugins.md#systemauslastung). |

## Accesibilidad {#bedienungshilfen}

Manta sigue **Ajustes del Sistema › Accesibilidad**, sin interruptores propios:

- **VoiceOver**: todos los botones, campos, listas y diálogos tienen nombre en tu idioma. Las filas de archivos se leen completas, p. ej. «README.md, Texto, 214 bytes, modificado Hoy 09:41, seleccionado», incluido el estado de la nube y de Git. Al moverte con las flechas, VoiceOver lee la fila actual. Los mensajes como «3 ítems copiados» y los errores se anuncian.
- **Reducir movimiento** (Pantalla): sin animaciones: se desactivan el destello de un favorito, el pulso del indicador de subida y los fundidos.
- **Aumentar contraste** (Pantalla): bordes y separadores más marcados, texto secundario más oscuro (más claro en modo oscuro) y un anillo de foco más visible.
- **Reducir transparencia** (Pantalla): sin oscurecimientos translúcidos detrás de los diálogos.

Manta aplica los cambios al instante, sin reiniciar. Incluso sin estos ajustes, el texto tiene un contraste mínimo de 4,5:1. Para usarlo solo con teclado, consulta [Atajos de teclado](18-tastenkuerzel.md#tastatur).

## Dónde guarda Manta sus ajustes {#dateien-speicherort}

Todo está en `~/Library/Application Support/Manta/`:

| Archivo | Contenido |
|---|---|
| `settings.json` | Ajustes generales |
| `favorites.json` | Favoritos y conexiones guardadas (sin contraseñas: esas están en el Llavero) |
| `session.json` | Paneles, pestañas y anchuras del último cierre |
| `plugins.json` | Qué plugins están activados, estado de los paneles |
| `plugin-data/` | Datos de los plugins, como `actions.json` |
| `update-state.json` | Canal de actualización, última búsqueda, versión omitida |
| `default-viewer.json` | Gestor de archivos predeterminado y la app anterior |
| `folder-views.json` | Visualización recordada por carpeta (con **Recordar la visualización por carpeta**) |
| `transfers.json` | Transferencias interrumpidas pendientes de reanudar (sin contraseñas); consulta [Transferencias](05-dateien.md#neustart) |

Las miniaturas se guardan aparte, en `~/Library/Caches/Manta/thumbs`, y se pueden eliminar en cualquier momento.

> [!TIP]
> Para trasladar Manta a otro Mac, copia esta carpeta mientras Manta no está abierto.
