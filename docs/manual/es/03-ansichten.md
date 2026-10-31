# Visualizaciones {#ansichten}

Como en Finder, cada panel puede mostrar sus ítems como **Iconos**, como **Lista**, en **Columnas** o como **Galería**. Cambia de visualización con **⌘1** a **⌘4** o con el selector de la barra de ruta.

![Las cuatro visualizaciones comparadas](images/views-overview.png)

## Cambiar la visualización {#wechseln}

| Visualización | Atajo | Comando | Ideal para… |
|---|---|---|---|
| Iconos | **⌘1** | **Como iconos** | Ver de un vistazo imágenes y carpetas con pocos ítems |
| Lista | **⌘2** | **Como lista** | Ordenar por tamaño, fecha o permisos |
| Columnas | **⌘3** | **Como columnas** | Recorrer árboles de carpetas profundos sin perder de vista la ruta |
| Galería | **⌘4** | **Como galería** | Ver imágenes, vídeos y documentos en una vista previa grande |

Hay tres maneras:

- **Teclado**: **⌘1** a **⌘4** actúan sobre el panel activo.
- **Barra de ruta**: a la derecha de la barra de ruta de cada panel está el selector **Visualización** con cuatro iconos. Si el panel es estrecho, se convierte en un único botón con un menú.
- **Paleta de comandos** (**⇧⌘P**): escribe, por ejemplo, “columnas” o “galería”. Los comandos están en el grupo **Visualización**.

La visualización pertenece al **panel**: todas las pestañas de un panel la comparten, y Manta la recuerda hasta el siguiente arranque. Dos paneles pueden mostrar visualizaciones distintas, por ejemplo Columnas a la izquierda y Lista a la derecha.

La selección (clic, **⌘** + clic, **⇧** + clic, **⌘A**), la selección escribiendo, Vista Rápida, el renombrado, el menú contextual, arrastrar y soltar y las insignias de los plugins funcionan igual en todas las visualizaciones.

## Opciones de visualización {#optionen}

En las visualizaciones Iconos, Columnas y Galería, el botón **Opciones de visualización** (icono de reguladores) está junto al selector. Abre un pequeño menú:

- **Ordenar por**: **Nombre**, **Tamaño**, **Fecha de modificación** o **Permisos**, además de **Ascendente** o **Descendente**.
- **Tamaño de los iconos** (solo en la visualización Iconos): un regulador de 48 a 256 puntos.

![Selector de visualización y Opciones de visualización en la barra de ruta](images/view-options.png)

La visualización Lista no tiene este botón: ahí ordenas con los encabezados de columna. En un panel estrecho, las opciones están en el menú del selector.

## Visualización Lista {#liste}

La lista muestra las columnas **Nombre**, **Tamaño**, **Fecha de modificación** y **Permisos**.

- **Ordenar**: haz clic en el encabezado de una columna. Un segundo clic invierte el orden. Las carpetas van siempre primero; las apps y otros paquetes se ordenan como archivos, como en el Finder.
- **Anchura de columna**: arrastra el borde izquierdo de una columna.
- **Tamaños de carpeta**: con el plugin [Tamaños de carpeta](14-plugins.md#ordnergroessen), la columna **Tamaño** muestra también el tamaño real de las carpetas.

![Visualización Lista ordenada por tamaño](images/view-list.png)

## Visualización Iconos {#symbole}

La visualización Iconos muestra una cuadrícula de iconos grandes y miniaturas con el nombre debajo.

- **Cambiar el tamaño**: con el regulador de **Opciones de visualización**, manteniendo pulsada **⌘** mientras te desplazas o pellizcando en el trackpad. Cada panel tiene su propio tamaño de iconos.
- **Seleccionar varios**: arrastra un rectángulo de selección empezando en un espacio vacío. Con **⌘** o **⇧** se conserva la selección anterior.
- **Teclado**: ← → ↑ ↓ recorren la cuadrícula; **Re Pág**/**Av Pág**, de página en página.

## Visualización Columnas {#spalten}

En la visualización Columnas, cada carpeta de la ruta es una columna propia. Al seleccionar una carpeta, la columna de su derecha muestra su contenido.

- **Teclado**: ↑ ↓ seleccionan en la columna actual, **→** abre la carpeta seleccionada como columna siguiente y **←** vuelve a la columna de la izquierda.
- **Columna de vista previa**: al seleccionar un archivo, la última columna muestra una vista previa con su miniatura o icono y la **Información**: **Tipo**, **Tamaño**, **Modificado**, **Permisos** y, en los alias, el **Original**. Con varios ítems seleccionados indica, por ejemplo, “3 ítems seleccionados”.
- **Anchura de columna**: arrastra el borde derecho de una columna (por el tirador de abajo). Haz doble clic en el borde para restablecer la anchura.
- Las carpetas vacías muestran “No hay ítems”. Si las columnas son más anchas que el panel, la visualización se desplaza lateralmente y mantiene visible la columna activa.

Si con **⌘↑** sales de la primera columna, o saltas mediante la barra lateral o la barra de ruta a una carpeta que no está en las columnas mostradas, las columnas empiezan de nuevo en esa carpeta.

![Visualización Columnas](images/view-columns.png)

## Visualización Galería {#galerie}

La galería muestra el ítem seleccionado en grande, con su nombre y debajo una línea con el tipo, el tamaño y la fecha de modificación. En la parte inferior hay una **tira de imágenes** con todos los ítems de la carpeta.

- Haz clic en un ítem de la tira o recórrelos con ← → para verlo en grande. **Re Pág**/**Av Pág** saltan el ancho de una tira.
- Si no hay nada seleccionado, la galería muestra el primer ítem.
- Las imágenes pequeñas se amplían como máximo al doble de su tamaño para que no se pixelen.

![Visualización Galería con tira de imágenes](images/view-gallery.png)

## Miniaturas {#vorschaubilder}

En las visualizaciones Iconos, Columnas y Galería, Manta muestra miniaturas. Se generan en segundo plano, así que la visualización sigue respondiendo con fluidez; hasta que una miniatura está lista, ves el icono del tipo de archivo. Sin miniatura, los archivos muestran el icono que macOS tiene para su tipo (también en las visualizaciones Lista y Columnas) y las apps (`.app`) el suyo propio, como en el Finder (se puede desactivar en **Ajustes**: **Iconos de archivo de macOS**).

- **Las imágenes** (PNG, JPEG, GIF, BMP, WebP) las lee el propio Manta y las gira según la orientación de la cámara (EXIF).
- **Los PDF, vídeos, HEIC, documentos**, etc. los proporciona macOS (como en Vista Rápida). Los archivos sin vista previa conservan su icono de tipo.
- Las miniaturas terminadas se guardan en `~/Library/Caches/Manta/thumbs`. Cuando un archivo cambia, se genera una miniatura nueva. Puedes eliminar esa carpeta cuando quieras.

### Miniaturas de archivos del servidor {#server-vorschaubilder}

Para los archivos **en servidores** (SFTP, FTP, S3, contenedores), Manta solo muestra por omisión el icono del tipo de archivo. Si también quieres miniaturas ahí, activa **Cargar miniaturas de archivos del servidor** en **Ajustes** → **General** → **Archivos**. Manta descargará entonces en segundo plano las imágenes pequeñas (menos de 5 MB). Los archivos más grandes conservan su icono.

> [!NOTE]
> Las miniaturas del servidor consumen datos y algo de tiempo. En conexiones lentas o con tarifa por consumo, deja la opción desactivada.

## Recordar la visualización por carpeta {#pro-ordner}

Por omisión, la visualización pertenece al **panel**: al cambiar de carpeta, la visualización no cambia.

Si activas **Recordar la visualización por carpeta** en **Ajustes** → **General** → **Archivos** (desactivado por omisión), cada carpeta se abre en la visualización en la que se mostró por última vez: tus fotos siempre como galería y tu proyecto como lista. Esto también funciona con carpetas de servidores y de archivos comprimidos. Las carpetas para las que no has elegido ninguna visualización mantienen la visualización actual del panel.

Manta recuerda las últimas 2000 carpetas en `~/Library/Application Support/Manta/folder-views.json`.

## Consulta también

- [Vista Rápida y vista previa](06-schnellansicht.md)
- [La barra de ruta](04-navigation.md#pfadleiste)
- [Ajustes](16-einstellungen.md#dateien)
- [Atajos de teclado](18-tastenkuerzel.md#ansichten)
