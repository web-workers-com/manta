# Gestionar archivos {#dateien}

Copiar, mover, renombrar, eliminar y crear carpetas: en tu Mac, en servidores, en contenedores y entre todos ellos. Lee sobre todo la sección sobre [eliminar](#loeschen): en tu Mac, lo eliminado va a la Papelera; en servidores, eliminar es definitivo.

## Copiar archivos a otro panel {#kopieren}

1. En el **panel de destino**, abre la carpeta a la que deben ir los archivos.
2. En el **panel activo**, selecciona los archivos y carpetas.
3. Pulsa **F5** (**Copiar al otro panel…**).
4. Comprueba en el diálogo qué va a dónde y elige qué ocurre con los nombres coincidentes (consulta [Conflictos de nombres](#konflikte)).
5. Haz clic en **Copiar**.

La copia se realiza en segundo plano. Puedes seguir trabajando de inmediato.

![Diálogo de copia con destino y opción para conflictos](images/copy-dialog.png)

**Mover** funciona igual con **F6** (**Mover al otro panel…**). Los ítems movidos desaparecen de su ubicación anterior.

Ambos comandos están también en la barra de herramientas (“Copiar al panel derecho”, “Mover al panel izquierdo”…) y en el menú contextual (clic derecho).

### ¿Qué panel es el destino? {#ziel}

Con dos paneles, es el otro. Con tres o más, es el otro panel que estuvo activo más recientemente (con una insignia **→**; consulta [Panel de destino](02-bereiche-tabs.md#zielbereich)). En **A**, dentro del diálogo, puedes elegir cualquier otro panel.

Manta evita errores típicos:

- “Ambos paneles muestran la misma carpeta — abre otra carpeta en el otro panel”
- “No se puede copiar una carpeta dentro de sí misma”

### Entre ubicaciones distintas {#orte}

Copiar y mover funcionan entre todas las ubicaciones: de tu Mac a un servidor SFTP, de un bucket de S3 a un contenedor de Docker, de un archivo comprimido al escritorio. Manta lo hace por sí mismo; no necesitas herramientas adicionales.

Desde ubicaciones de **solo lectura** (archivos comprimidos, contenedores detenidos), mover solo copia. Manta te lo indica, por ejemplo: “Los archivos comprimidos son de solo lectura: los ítems se extraen (se copian), no se mueven”.

## Conflictos de nombres {#konflikte}

Si en el destino ya existe un ítem con el mismo nombre, decide el ajuste **Si un ítem ya existe** del diálogo:

| Opción | Qué ocurre |
|---|---|
| **Preguntar** | Manta se detiene en cada ítem que ya existe y te pregunta (opción por omisión). |
| **Reemplazar** | Sobrescribe siempre el ítem existente. |
| **Omitir** | No toca los ítems existentes. |
| **Reemplazar si es más reciente** | Sobrescribe solo si el origen es más reciente o tiene otro tamaño. |
| **Conservar ambos** | La copia recibe un nombre nuevo (“nombre 2”). |

Las demás opciones se aplican a toda la tarea sin preguntar. Define la opción por omisión en **Ajustes** → **General** → **Copiar y mover**.

### Preguntar {#nachfragen}

Con **Preguntar**, la tarea se detiene en el primer ítem que ya existe en el destino. Manta muestra *“nombre” ya existe en “carpeta”* con ambos ítems (tamaño, fecha de modificación y una etiqueta **Más reciente** en el más nuevo) y tú eliges:

| Botón | Qué ocurre |
|---|---|
| **Conservar ambos** | La copia recibe un nombre nuevo (“nombre 2”). Es la opción por omisión: **↩** la elige. |
| **Reemplazar** | Se sobrescribe el ítem existente. |
| **Reemplazar si es más reciente** | Sobrescribe solo si el origen es más reciente; si no, se conserva el existente. |
| **Omitir** | El ítem existente se conserva y este no se transfiere (al mover, se queda donde estaba). |
| **Cancelar** (**esc**) | La tarea se detiene. Lo que ya se haya transferido se queda en el destino. |

Si ya existe una **carpeta** con ese nombre, Manta ofrece **Combinar** (como Finder): el contenido va a la carpeta existente y, por cada archivo que ya esté ahí, se te pregunta uno por uno. Además están **Conservar ambos** (“fotos 2”) y **Omitir**. Las subcarpetas siempre se combinan.

Con **Aplicar a todos los conflictos restantes de esta tarea**, la respuesta vale para el resto de la tarea y Manta no vuelve a preguntar. Si varias tareas preguntan a la vez, se van turnando. Las demás transferencias siguen en curso mientras una espera; la lista de transferencias la muestra como “Esperando tu decisión”. Si cierras Manta con una pregunta abierta, la tarea vuelve a preguntar cuando se reanuda.

## Seguir las transferencias {#uebertragungen}

**⌥⌘L** (**Mostrar transferencias**) o el botón **Transferencias** de la barra de herramientas muestra todas las tareas en curso: copiar, mover, eliminar, extraer y comprimir.

- Cada tarea muestra su progreso, su velocidad y el tiempo restante.
- **Pausar** detiene temporalmente una tarea y **Reanudar** la deja continuar (consulta más abajo).
- **Detener** cancela una tarea. Los archivos copiados a medias se eliminan.
- **Borrar terminadas** limpia la lista.
- Si algo ha fallado, la tarea muestra “N problemas”. El aviso “Terminado con problemas” también remite aquí.

Cuando ocultas el panel, la barra de estado muestra abajo a la derecha una pequeña píldora con un anillo de progreso, para que siempre veas cuánto le falta a una tarea, también para archivos soltados desde otras apps o copiados a carpetas en la nube:

- una tarea: “Copiando 3 ítems… 42 %”, *Moviendo “Vacaciones.mov”… 80 %*,
- varias: “2 transferencias · 37 %”,
- en pausa o en espera: “… · en pausa”, “… · en espera”, “… · esperando conexión”,
- después, **Terminado** (o **Detenido**) durante unos segundos,
- si algo ha fallado, la píldora roja (“Error en la transferencia”, “Terminado con errores”) permanece hasta que haces clic en ella.

Haz clic en la píldora para abrir las transferencias; su descripción emergente enumera cada tarea en curso con su progreso y el tiempo restante.

Cuando copias a una carpeta en la nube (Google Drive, iCloud Drive, OneDrive, Dropbox), la transferencia termina en cuanto los archivos están en tu Mac; la app del proveedor los sube después. La barra de estado también lo muestra, por ejemplo “Google Drive está subiendo 2 ítems” o, en rojo, “Google Drive: 1 error de subida” (consulta [Almacenamiento en la nube](09-cloud.md#abzeichen)).

![Transferencias con una tarea en curso](images/transfers-panel.png)

## Pausar y reanudar transferencias {#pausieren}

Cualquier tarea de copia, traslado o eliminación se puede pausar y reanudar más tarde en el mismo punto, incluso en mitad de un archivo grande.

- **Una tarea**: haz clic en el icono de pausa de su fila (**Pausar**). La fila muestra entonces, por ejemplo, “En pausa · 1,2 GB de 3,0 GB”. El icono de reproducción (**Reanudar**) la deja continuar.
- **Todas a la vez**: en la parte superior de **Transferencias** encontrarás **Pausar todo** y **Reanudar todo** siempre que haya algo que pausar o reanudar.

Una tarea reanudada se salta los archivos terminados y continúa el archivo en curso donde se detuvo (“Reanudando desde 1,2 GB”).

**Comprimir** no se puede pausar.

### Cuando se pierde la conexión {#verbindung-weg}

Si se cae la red o el servidor desaparece un momento, Manta no abandona la tarea. **Espera** y lo vuelve a intentar por sí mismo:

- La fila muestra “Esperando conexión… (intento 3, siguiente en 8 s)”. Las pausas se van alargando (1, 2, 4… segundos, como máximo 1 minuto); Manta sigue intentándolo.
- En cuanto tu Mac vuelve a estar conectado, la tarea continúa de inmediato, donde se detuvo.
- Si no quieres esperar, haz clic en **Reintentar ahora** (icono de flecha).
- Cuando tu Mac no tiene red en absoluto, **Transferencias** muestra el aviso **No hay conexión a internet**: “Las transferencias continuarán donde se detuvieron en cuanto vuelva la conexión.”

Una tarea en espera se puede pausar o detener como cualquier otra.

![Transferencias: una tarea en pausa y otra en espera, aviso “No hay conexión a internet”](images/transfers-paused-waiting.png)

### Archivos parciales (`.manta-part`) {#teildateien}

Mientras se copia un archivo, Manta lo escribe primero en un **archivo parcial oculto** junto al destino: `film.mov` se convierte en `.film.mov.manta-part`. Solo cuando el archivo está completo recibe su nombre real. Así, el destino nunca contiene medio archivo con el nombre real.

- Solo ves los archivos parciales cuando se muestran los archivos ocultos (**⇧⌘.**).
- No elimines un archivo parcial mientras la tarea está en pausa o en espera; si lo haces, ese archivo empieza de nuevo.
- **Detener** y **Descartar** (consulta más abajo) los eliminan por ti.
- Las subidas a **S3** y **Google Drive** no necesitan archivo parcial: el servidor mantiene invisible el archivo incompleto hasta que está completo.

### Después de cerrar la app o de un fallo {#neustart}

Manta recuerda las transferencias sin terminar aunque cierres la app, se produzca un fallo o el Mac se reinicie. En el siguiente arranque pregunta:

**¿Reanudar las transferencias interrumpidas?**, por ejemplo “2 transferencias no terminaron la última vez.” Debajo enumera las tareas con su progreso y, si hace falta, qué servidor falta (“requiere files.example.com (SFTP)”).

- **Reanudar** (o ↩): las tareas continúan donde se detuvieron.
- **Descartar**: las tareas se descartan y sus archivos parciales se eliminan, en la medida en que Manta puede acceder a ellos. En un servidor que no está conectado queda un archivo parcial; puedes eliminarlo tú más tarde.
- **esc** (cerrar la ventana): decide más tarde. Manta vuelve a preguntar en el siguiente arranque.

![La pregunta “¿Reanudar las transferencias interrumpidas?”](images/resume-prompt.png)

#### “Requiere una conexión” {#benoetigt-verbindung}

Cuando una tarea reanudada necesita un servidor al que todavía no estás conectado, su fila muestra “Requiere una conexión con files.example.com (SFTP)” y el botón **Reconectar**.

- **Los favoritos de servidor** con contraseña recordada (o con ssh-agent) se reconectan solos en segundo plano; no tienes que hacer nada.
- Si no, haz clic en **Reconectar**. Manta abre el diálogo de conexión con los datos del servidor; introduce la contraseña. En cuanto se establece la conexión, la tarea continúa.

Manta **nunca** guarda contraseñas ni frases de contraseña para esto: la lista de transferencias interrumpidas (`transfers.json`) solo contiene servidores, nombres de usuario y rutas.

#### Notas de “copiado de nuevo desde el principio” {#von-vorn}

A veces Manta no puede continuar un archivo de forma segura y lo vuelve a copiar. La tarea muestra entonces “1 nota”; haz clic en ella para desplegar las notas:

| Nota | Qué ha ocurrido |
|---|---|
| “el origen ha cambiado desde la interrupción; se ha copiado de nuevo desde el principio” | El archivo original tiene ahora otro tamaño u otra fecha de modificación. Continuar mezclaría lo antiguo con lo nuevo. |
| “este servidor no puede continuar archivos parciales; se ha copiado de nuevo desde el principio” | El destino no permite añadir datos a un archivo sin terminar. |
| “el archivo parcial era más grande que el origen; se ha copiado de nuevo desde el principio” | El archivo parcial no se corresponde con el origen. |
| “la sesión de carga ha caducado; se ha subido de nuevo desde el principio” | S3 o Google Drive descartaron entretanto la subida sin terminar. |

### Limitaciones {#pausieren-grenzen}

- **Comprimir** no se puede pausar y no sobrevive a un reinicio.
- Las tareas **desde archivos comprimidos** (extraer) se pueden pausar, pero no se reanudan después de cerrar la app.
- Sin la contraseña correcta, una tarea en un servidor espera hasta que te conectes.

## Arrastrar y soltar {#drag-drop}

### Entre paneles

1. Selecciona ítems y arrástralos a otro panel.
2. Suéltalos. Manta pregunta: en el diálogo eliges **Copiar** o **Mover**, el destino y qué ocurre si hay conflictos de nombres. **Copiar** está preseleccionado, también dentro del mismo volumen de tu Mac, donde Finder movería. Así nada desaparece de su carpeta por accidente. **↩** confirma la opción preseleccionada.
3. Mantén pulsada **⌘** al soltar para preseleccionar **Mover**. **⌥** preselecciona **Copiar** de forma explícita (y tiene prioridad sobre ⌘).

Una insignia junto al puntero muestra qué está preseleccionado (“Copiar 3 ítems aquí · mantén ⌘ para mover”; con ⌘: “Mover 3 ítems aquí · mantén ⌥ para copiar”). Arrastrar desde un archivo comprimido extrae. Entre paneles, los ítems van a la carpeta que muestra el panel.

Si no quieres que se te pregunte cada vez, elige “Copiar siempre” o “Mover siempre” en **Al soltar archivos**, dentro de [Ajustes](16-einstellungen.md#kopieren-bewegen). La tarea empieza entonces sin diálogo; **⌘** sigue moviendo y **⌥** sigue copiando para ese arrastre concreto. No afecta a lo que se suelta en la **Papelera** ni en **Favoritos**.

### De Manta a otras apps {#drag-raus}

Puedes arrastrar archivos de Manta a otras apps: al escritorio, a una ventana de Finder, a Mail o a la subida de archivos de un navegador.

- **Los archivos locales** se copian, también cuando el destino está en el mismo volumen. Mantén pulsada **⌘** para moverlos.
- **Los archivos de servidores, contenedores o archivos comprimidos** se descargan en el destino al soltarlos. Con archivos grandes, puede tardar un rato.

Arrastra los ítems más allá del borde de la ventana de Manta; a partir de ahí se encarga macOS. Un mensaje te indica cuándo empieza una descarga desde un servidor y cuándo ha terminado o ha fallado. Si la otra app ha movido los archivos, Manta actualiza la carpeta de la que salieron.

### De otras apps a Manta {#drag-rein}

Suelta archivos de Finder, del escritorio o de otra app en un panel:

- En un espacio vacío del panel: van a la carpeta que muestra el panel.
- Sobre una carpeta (en todas las visualizaciones; en Columnas, también sobre una columna): van a esa carpeta. Se resalta mientras pasas por encima.
- Se aplican las mismas reglas que entre paneles: Manta pregunta con **Copiar** preseleccionado (con **⌘**: **Mover**), salvo que hayas elegido “Copiar siempre” o “Mover siempre”.

La tarea aparece en Transferencias, donde puedes pausarla. Los adjuntos de Mail y las fotos de Fotos todavía no se pueden soltar en Manta.

### En Favoritos

Arrastra una carpeta a la sección **Favoritos** de la barra lateral para tenerla siempre a mano. Consulta [Favoritos y barra lateral](07-favoriten-seitenleiste.md#favoriten-ziehen).

### En la Papelera

Arrastra ítems a **Papelera**, en la barra lateral: los ítems locales van a la papelera sin preguntar; los ítems de servidores se eliminan definitivamente tras preguntar. Consulta [Papelera](07-favoriten-seitenleiste.md#papierkorb).

## Renombrar {#umbenennen}

1. Selecciona **un** ítem.
2. Pulsa **F2** (**Renombrar…**) o elige **Renombrar…** en el menú contextual.
3. Escribe el nombre nuevo y pulsa ↩.

**Renombrar con un clic**, como en Finder: vuelve a hacer clic en un ítem que ya esté seleccionado y espera un momento. En la vista de lista el nombre pasa a ser editable en la propia fila (se selecciona el nombre sin la extensión); ↩ o un clic fuera lo renombra y Esc cancela. En las demás vistas se abre el diálogo **Renombrar…**. Un doble clic sigue abriendo el ítem.

Manta comprueba el nombre al instante: no puede estar vacío, contener “/” ni existir ya en la carpeta.

**Renombrar muchos ítems a la vez**: selecciona varios y pulsa **F2**, o pulsa **⇧⌘R** (**Renombrar en lote…**). El plugin ofrece reglas, una vista previa en directo y **Deshacer**. Consulta [Renombrar en lote](14-plugins.md#umbenennen).

## Nueva carpeta {#neuer-ordner}

1. Pulsa **⇧⌘N** (**Nueva carpeta…**).
2. Escribe un nombre (se sugiere “carpeta sin título”) y haz clic en **Crear**.

## Nuevo archivo {#neue-datei}

1. Pulsa **⌥⌘N** (**Nuevo archivo…**) o elige **Nuevo archivo…** en el menú **Archivo** o en el menú contextual de un panel.
2. Escribe un nombre (se sugiere “archivo sin título.txt”) y haz clic en **Crear**. Manta crea un archivo vacío y lo selecciona.

**Dentro de una carpeta**: si haces clic derecho en una carpeta, las opciones son **Nueva carpeta en “…”** y **Nuevo archivo en “…”**; la carpeta o el archivo se crean dentro de esa carpeta y el panel se queda donde está. **⇧⌘N** y **⌥⌘N** siguen creando en la carpeta mostrada.

## Eliminar {#loeschen}

Manta elimina como Finder:

| Dónde | ⌘⌫ (**Trasladar a la papelera**) | ⌥⌘⌫ (**Eliminar inmediatamente…**) |
|---|---|---|
| Tu Mac (también unidades externas y carpetas en la nube) | Traslada los ítems a la **Papelera**, sin preguntar. **⌘Z** (**Deshacer**) o la opción **Devolver** de Finder los restauran. | Elimina definitivamente tras preguntar. |
| Servidores (SFTP, FTP, FTPS), S3, contenedores | No hay papelera: elimina definitivamente tras preguntar. | Elimina definitivamente tras preguntar. |
| Archivos comprimidos | Solo lectura: no se puede eliminar nada. | Solo lectura. |

Para trasladar ítems a la papelera:

1. Selecciónalos.
2. Pulsa **⌘⌫** o la tecla de borrar hacia delante **⌦**, o elige **Trasladar a la papelera** en el menú contextual o en la barra de herramientas, o arrástralos a **Papelera** en la barra lateral.

Un mensaje lo confirma (“Se han trasladado 3 ítems a la papelera”). Para recuperar los ítems, pulsa **⌘Z** (**Deshacer**): Manta los devuelve a donde estaban; consulta [Deshacer](#widerrufen). Para ítems más antiguos o eliminados desde otra app, abre la papelera en Finder, selecciónalos y elige **Devolver**.

Para eliminar definitivamente:

1. Selecciona los ítems.
2. Pulsa **⌥⌘⌫** o elige **Eliminar inmediatamente…** en la paleta de comandos.
3. Manta pregunta: *¿Eliminar “nombre”?* o “¿Eliminar 3 ítems?”, con la indicación de que los ítems no van a la papelera y de que la acción no se puede deshacer.
4. Haz clic en **Eliminar**.

En un **servidor**, en un **bucket de S3** o en un **contenedor** no hay papelera. **⌘⌫**, el menú contextual (ahí, **Eliminar inmediatamente…**) y el botón de la barra de herramientas preguntan siempre antes, y el diálogo indica en rojo que esa ubicación no tiene papelera y que allí eliminar es definitivo. Las carpetas se eliminan con todo su contenido.

Para abrir la propia papelera, haz clic en **Papelera** en la barra lateral. Dentro, **⌘⌫** (ahí: **Eliminar inmediatamente…**) elimina definitivamente tras preguntar, y **⇧⌘⌫** (**Vaciar papelera…**) la vacía. Consulta [Papelera](07-favoriten-seitenleiste.md#papierkorb).

> [!CAUTION]
> En las **carpetas en la nube** (iCloud Drive, Google Drive, OneDrive, Dropbox), un ítem que trasladas a la papelera desaparece también de la nube en cuanto tu Mac se sincroniza. **Devolver** suele recuperarlo; algunos proveedores también tienen una papelera en su sitio web. Consulta [Almacenamiento en la nube](09-cloud.md).

## Deshacer {#widerrufen}

**⌘Z** (**Deshacer**) revierte tu última operación con archivos, como en Finder, y **⇧⌘Z** (**Rehacer**) la vuelve a hacer. Ambos están en el menú **Edición** y en la paleta de comandos. Manta recuerda hasta 100 pasos mientras no lo cierres.

| Lo que hiciste | Lo que hace **⌘Z** |
|---|---|
| **Trasladar a la papelera** (**⌘⌫**, o arrastrar a **Papelera** en la barra lateral) | Devuelve los ítems a donde estaban. |
| Renombrar (**Renombrar…**, renombrar con un clic, la ventana de información) | Recupera el nombre anterior, también en servidores. |
| **Nueva carpeta…** o **Nuevo archivo…** en tu Mac | Traslada el ítem nuevo a la papelera, como Finder, junto con todo lo que hayas puesto dentro. **⇧⌘Z** lo recupera. |
| **Nueva carpeta…** o **Nuevo archivo…** en un servidor, en S3 o en un contenedor | No hay papelera: Manta elimina el ítem nuevo, pero solo mientras siga vacío. Si no, avisa de que no ha podido eliminarlo y lo deja donde está. |

No se puede deshacer copiar y mover (**F5**, **F6**, arrastrar y soltar) ni nada definitivo: **Eliminar inmediatamente…**, eliminar en servidores y **Vaciar papelera…**. **Renombrar en lote…** tiene su propio **Deshacer**; consulta [Renombrar en lote](14-plugins.md#umbenennen).

Si mientras tanto el lugar original vuelve a estar ocupado (hay un ítem con el mismo nombre), Manta no reemplaza nada. Avisa con “No se ha podido devolver ‘nombre’” y conserva el paso, así que **⌘Z** vuelve a funcionar en cuanto el lugar quede libre.

- En un **campo de texto** (filtro, ruta, el nombre que estás cambiando), **⌘Z** y **⇧⌘Z** deshacen lo que escribes, no las operaciones con archivos. Mientras el **terminal** tiene el foco, **⌘Z** pertenece al terminal.
- Sin [Acceso total al disco](07-favoriten-seitenleiste.md#papierkorb-zugriff), Manta devuelve los ítems de la papelera a través de Finder, igual que con **Vaciar papelera…**: macOS pregunta una vez si Manta puede controlar Finder.

## Etiquetas {#tags}

Manta lee y asigna las mismas etiquetas que el Finder (se guardan con el archivo, así que el Finder, Spotlight y otras apps las ven). Los ítems etiquetados muestran puntos de color junto a su nombre en todas las visualizaciones: en Lista y Columnas detrás del nombre, en Iconos y Galería delante. Las etiquetas funcionan con los ítems de este Mac, incluidos iCloud Drive y otras carpetas en la nube; los servidores y los archivos comprimidos no tienen.

**Añadir o eliminar una etiqueta:**

1. Selecciona los ítems y abre el menú contextual.
2. Elige **Etiquetas**. Los siete colores están arriba: un clic añade la etiqueta a todos los ítems seleccionados, o la elimina si todos la tienen ya (una marca de verificación indica que todos la tienen; un guion, que solo algunos). Tus otras etiquetas aparecen debajo.
3. Para crear una etiqueta nueva, escribe su nombre en **Nueva etiqueta…** y pulsa **↩**.

Las etiquetas estándar llevan los nombres que usa el Finder en tu Mac (por ejemplo, “Rojo” en un Mac en español, “Red” en uno en inglés).

**Encontrar ítems etiquetados:** La sección **Etiquetas** de la barra lateral muestra tus etiquetas como las muestra el Finder (si Manta no puede leer la lista del Finder, las etiquetas que ha visto en los archivos). Un clic muestra en el panel activo todo lo que tiene esa etiqueta, encontrado por Spotlight. En el campo de filtro, `tag:Rojo` muestra los ítems etiquetados de la carpeta actual (consulta [Filtrar archivos](04-navigation.md#filtern)).

## Obtener información {#informationen}

**⌘I** (**Obtener información**, también en el menú contextual y en el menú Archivo) abre una ventana de información de la selección, como en el Finder: una ventana por ítem (si vuelves a pedirla, pasa al frente) y una ventana de resumen para varios ítems. Puede haber hasta seis ventanas de información abiertas; al abrir una nueva se cierra la más antigua. Son ventanas normales: puedes seguir trabajando en la ventana principal.

Una ventana de información muestra:

- el **nombre**: edítalo y pulsa **↩** para renombrar el ítem,
- **Clase**, **Tamaño** (las carpetas y los ítems múltiples se calculan en segundo plano), **Ubicación**, **Creado**, **Modificado** y **Última apertura** (la fecha de Spotlight),
- **Etiquetas**: elimina una con su ×, añade una con **Añadir etiqueta**,
- **Bloqueado** (el ítem no se puede modificar, renombrar ni trasladar a la papelera) y **Oculto**,
- **Comentarios**: el comentario de Spotlight asignado en el Finder (Manta lo muestra pero no puede cambiarlo: macOS no ofrece ninguna forma de hacerlo salvo mediante scripts del Finder),
- **Abrir con**: la app por omisión, y **Abrir con** para abrir el archivo una vez con otra app. macOS no ofrece a otras apps ninguna forma pública de cambiar la app de un único archivo, así que no hay “Cambiar todo…”; para eso, usa el Finder,
- **Compartir y permisos**: lectura, escritura y ejecución para propietario, grupo y todos, además del valor octal (por ejemplo `644`). En este Mac y en servidores SFTP puedes cambiarlos ahí mismo (chmod); en otros lugares se muestran como solo lectura.

Los ítems de servidores, contenedores y archivos comprimidos muestran lo que indica el servidor (tamaño, fecha, permisos).

## Ubicaciones de solo lectura {#schreibgeschuetzt}

En los archivos comprimidos, en los contenedores detenidos y en los contenedores sin shell no puedes cambiar nada. Una etiqueta **Solo lectura** (o **Detenido**) en la barra de ruta lo indica. Copiar hacia fuera con **F5** o arrastrando funciona siempre.

Como el Finder, **Aplicaciones** muestra también las apps de Apple (Calculadora, Mail…) y **Utilidades** las de macOS. Están en el volumen del sistema protegido: puedes abrirlas y copiarlas, pero no renombrarlas, moverlas ni eliminarlas.

## Consulta también

- [Archivos comprimidos](10-archive.md): comprimir y extraer.
- [Terminal](11-terminal.md#dateiaktionen): ver cada acción de archivo como comando de shell.
