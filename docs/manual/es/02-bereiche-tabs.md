# Paneles y pestañas {#bereiche-tabs}

Manta muestra tus archivos en **paneles** uno al lado del otro: dos, tres o tantos como quepan en la ventana. Cada panel tiene sus propias pestañas, su barra de ruta y su historial.

![Tres paneles uno al lado del otro, con el panel activo resaltado](images/panes-three.png)

## El panel activo {#aktiver-bereich}

Siempre hay exactamente un panel **activo**: tiene un contorno marcado en el color de acentuación. Los atajos y comandos actúan sobre el panel activo.

- Haz clic en cualquier parte de un panel para activarlo.
- **⇥** (Tab) pasa al panel siguiente (después del último, vuelve al primero).
- **⌥⌘←** y **⌥⌘→** activan el panel de la izquierda o de la derecha.

## Dividir un panel {#teilen}

1. Activa el panel junto al que debe aparecer el nuevo.
2. Pulsa **⌘D** (**Dividir panel**).

El panel nuevo aparece a la derecha del activo y muestra la misma carpeta. También puedes hacer clic en el botón de dividir, a la derecha de la barra de ruta del último panel, o elegir **Dividir panel** en la paleta de comandos.

Cada panel necesita unos 280 puntos de anchura. Si la ventana es demasiado estrecha, Manta indica “No hay espacio para otro panel”. Ensancha la ventana o cierra un panel.

## Cerrar un panel {#schliessen}

- Pulsa **⇧⌘D** (**Cerrar panel**), o
- haz clic en la **×** a la derecha de la barra de ruta del panel.

El último panel no se puede cerrar.

> [!NOTE]
> Con un solo panel, **Copiar** y **Mover al otro panel**, **Comparar y sincronizar** y el cambio de panel están desactivados. La descripción emergente explica el motivo: “Se necesitan dos paneles – ⌘D divide el panel”.

## Cambiar la anchura de los paneles {#breite}

- Arrastra el separador entre dos paneles a la izquierda o a la derecha.
- Haz doble clic en un separador para que todos los paneles vuelvan a tener la misma anchura.

## El panel de destino: “el otro panel” {#zielbereich}

Copiar (**F5**) y Mover (**F6**) van siempre al **panel de destino**.

- Con **dos paneles**, es simplemente el otro.
- Con **tres o más**, es el panel que estuvo activo más recientemente. Mientras el panel activo tiene una selección, el panel de destino muestra un contorno tenue y una insignia **→** en su barra de ruta (“Copiar y Mover (F5 / F6) van a este panel”).

En el diálogo de copia puedes elegir otro panel en **A**. Consulta [Gestionar archivos](05-dateien.md#kopieren).

![Panel de destino con la insignia de flecha](images/panes-target-badge.png)

## Pestañas {#tabs}

Cada panel puede tener varias pestañas. Una pestaña muestra una carpeta: en tu Mac, en un servidor, en un contenedor o en un archivo comprimido.

| Acción | Cómo |
|---|---|
| Nueva pestaña | **⌘T** o el botón **+** de la barra de ruta, o **Nueva pestaña** en el menú del Dock (clic secundario en el icono de Manta en el Dock) |
| Cerrar pestaña | **⌘W** o la **×** de la pestaña (aparece al pasar el puntero) |
| Cambiar de pestaña | Haz clic en la pestaña |

La pestaña nueva muestra la misma carpeta que la actual, o la carpeta que hayas elegido en [**Las pestañas nuevas se abren en**](16-einstellungen.md#dateien). **Ir a la ruta…** (**⇧⌘G**) abre una carpeta directamente en una pestaña nueva (consulta [Navegación](04-navigation.md#gehe-zum-pfad)).

> [!NOTE]
> Como en Finder, **⌘T** abre una pestaña nueva. Antes de esta versión era **⇧⌘T**, y **⌘T** mostraba las transferencias; ahora eso es **⌥⌘L**.

En paneles muy estrechos, el botón **+** desaparece de la barra de ruta; **⌘T** sigue funcionando.

## Sesiones: todo vuelve {#sitzung}

Cuando cierras Manta y lo vuelves a abrir, se recupera lo siguiente:

- los paneles, con su orden y su anchura,
- las pestañas que muestran carpetas **locales**,
- qué panel estaba activo y cuál es el de destino.

Las pestañas con carpetas de servidores, contenedores o archivos comprimidos no se restauran, porque necesitarían una nueva conexión. Ábrelas desde tus [Favoritos](07-favoriten-seitenleiste.md).

Si una carpeta guardada ya no existe en el siguiente arranque (porque se expulsó su unidad, por ejemplo), la pestaña vuelve igualmente. Muestra un error con **Reintentar**: vuelve a conectar la unidad y haz clic en el botón, o cierra la pestaña o abre otra carpeta.

## Consulta también

- [Visualizaciones](03-ansichten.md): Iconos, Lista, Columnas, Galería.
- [Comparar y sincronizar](13-vergleichen-synchronisieren.md): igualar dos paneles.
