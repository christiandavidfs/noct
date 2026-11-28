import QtQuick
import QtQuick.Effects
import Quickshell
import Quickshell.Io
import Quickshell.Wayland

ShellRoot {
    PanelWindow {
        id: hud

        anchors {
            top: true
            bottom: true
            left: true
            right: true
        }
        color: "transparent"
        visible: false

        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
        WlrLayershell.namespace: "rakia-switcher"
        WlrLayershell.exclusiveZone: 0

        // La superficie vive en el monitor que tiene el foco al abrir.
        screen: {
            for (const s of Quickshell.screens) {
                if (s.name === hud.focusOut)
                    return s;
            }
            return Quickshell.screens.length > 0 ? Quickshell.screens[0] : null;
        }

        // ---------------------------------------------------------------- state
        property var windowsAll: []
        property var outputsAll: []
        property var mru: []
        property var list: []
        property string lastActiveId: ""
        property int selected: 0
        property bool pendingOpen: false
        property bool opening: false
        property bool captureWanted: false
        property bool reveal: false
        property string shotPath: ""
        property string pendingShot: ""
        property string focusOut: ""
        property int capX: 0
        property int capY: 0
        property int capW: 0
        property int capH: 0
        property var backdropClip: Qt.rect(0, 0, 0, 0)

        readonly property color accent: "#7CFFB2"
        readonly property color accentSoft: "#3E7F63"
        readonly property color ink: "#05060A"
        readonly property int cardW: Math.min(width * 0.44, 840)
        readonly property int cardH: Math.round(cardW * 0.62)

        signal commitRequested(string windowId)

        // -------------------------------------------------------------- helpers
        function runtimeDir() {
            const rd = Quickshell.env("XDG_RUNTIME_DIR");
            return (rd ? String(rd) : "/tmp") + "/rakia-switcher";
        }

        function shq(value) {
            return "'" + String(value).replace(/'/g, "'\\''") + "'";
        }

        function focusedOutput() {
            for (const o of outputsAll) {
                if (o.focused)
                    return o;
            }
            return outputsAll.length > 0 ? outputsAll[0] : null;
        }

        function windowById(id) {
            for (const w of windowsAll) {
                if (w.id === id)
                    return w;
            }
            return null;
        }

        function activeWindow() {
            const out = focusedOutput();
            if (out) {
                for (const w of windowsAll) {
                    if (w.focused && w.output === out.name)
                        return w;
                }
            }
            for (const w of windowsAll) {
                if (w.focused)
                    return w;
            }
            return null;
        }

        function trackFocus() {
            const active = activeWindow();
            if (!active)
                return;
            if (active.id === lastActiveId)
                return;
            lastActiveId = active.id;
            const next = [active.id];
            for (const id of mru) {
                if (id !== active.id && windowById(id))
                    next.push(id);
            }
            mru = next;
        }

        function orderedWindows() {
            const out = [];
            const seen = {};
            for (const id of mru) {
                const w = windowById(id);
                if (w && !seen[id]) {
                    out.push(w);
                    seen[id] = true;
                }
            }
            for (const w of windowsAll) {
                if (!seen[w.id]) {
                    out.push(w);
                    seen[w.id] = true;
                }
            }
            return out;
        }

        function computeLayout() {
            let minX = 0, minY = 0, maxX = 0, maxY = 0, first = true;
            for (const o of outputsAll) {
                const x = o.position.x;
                const y = o.position.y;
                const w = o.logical_size.width;
                const h = o.logical_size.height;
                if (first) {
                    minX = x;
                    minY = y;
                    maxX = x + w;
                    maxY = y + h;
                    first = false;
                } else {
                    minX = Math.min(minX, x);
                    minY = Math.min(minY, y);
                    maxX = Math.max(maxX, x + w);
                    maxY = Math.max(maxY, y + h);
                }
            }
            capX = minX;
            capY = minY;
            capW = maxX - minX;
            capH = maxY - minY;
        }

        function previewRect(w) {
            if (!w || !w.visible || shotPath === "")
                return null;

            const ratio = cardW / cardH;
            let rx = w.x - capX;
            let ry = w.y - capY;
            let rw = w.w;
            let rh = Math.min(w.h, Math.round(rw / ratio));

            // recorta desde arriba, que es donde está el título de la ventana
            ry += Math.min(26, Math.max(0, w.h - rh));

            if (rx < 0) {
                rw += rx;
                rx = 0;
            }
            if (ry < 0) {
                rh += ry;
                ry = 0;
            }
            if (rx + rw > capW)
                rw = capW - rx;
            if (ry + rh > capH)
                rh = capH - ry;
            if (rw <= 0 || rh <= 0)
                return null;
            return Qt.rect(rx, ry, rw, rh);
        }

        function advance(delta) {
            if (list.length === 0)
                return;
            selected = (selected + delta + list.length) % list.length;
            idleTimer.restart();
        }

        function closeHud() {
            idleTimer.stop();
            reveal = false;
            visible = false;
            pendingOpen = false;
            opening = false;
            captureWanted = false;
            shotPath = "";
            list = [];
        }

        function commit() {
            const w = list.length > 0 ? list[Math.max(0, Math.min(selected, list.length - 1))] : null;
            if (w)
                commitRequested(w.id);
            closeHud();
        }

        function showHud() {
            opening = false;
            visible = true;
            reveal = true;
            Qt.callLater(() => keyCatcher.forceActiveFocus());
            idleTimer.restart();
        }

        function openHud(startDirection) {
            if (visible) {
                advance(1);
                return;
            }
            if (opening)
                return;
            const direction = startDirection === undefined ? 1 : startDirection;
            if (windowsAll.length === 0 || outputsAll.length === 0) {
                pendingOpen = true;
                snapshotProcess.running = true;
                return;
            }

            const out = focusedOutput();
            focusOut = out ? out.name : "";
            computeLayout();
            if (out) {
                backdropClip = Qt.rect(
                    out.position.x - capX,
                    out.position.y - capY,
                    out.logical_size.width,
                    out.logical_size.height
                );
            }

            list = orderedWindows();
            if (list.length === 0)
                return;

            const active = activeWindow();
            let index = -1;
            if (active) {
                for (let i = 0; i < list.length; i++) {
                    if (list[i].id === active.id) {
                        index = i;
                        break;
                    }
                }
            }
            if (list.length > 1) {
                if (index >= 0)
                    selected = (index + (direction > 0 ? 1 : -1) + list.length) % list.length;
                else
                    selected = direction > 0 ? 1 % list.length : list.length - 1;
            } else {
                selected = 0;
            }

            shotPath = "";
            opening = true;
            captureWanted = true;
            pendingShot = runtimeDir() + "/shot-" + Date.now() + ".jpg";
            const dir = runtimeDir();
            const cmd = "mkdir -p " + shq(dir)
                + " && grim -s 1 -t jpeg -q 82 " + shq(pendingShot)
                + " ; ls -t " + shq(dir) + "/shot-* 2>/dev/null | tail -n +4 | xargs -r rm -f";
            captureProcess.command = ["sh", "-c", cmd];
            captureProcess.running = true;
        }

        function onSnapshot(raw) {
            try {
                const sep = raw.indexOf("\n---\n");
                const outs = JSON.parse(sep >= 0 ? raw.substring(0, sep) : "[]");
                const wins = JSON.parse(sep >= 0 ? raw.substring(sep + 5) : raw);
                outputsAll = outs;
                windowsAll = wins;
                trackFocus();
            } catch (e) {
                console.log("rakia-switcher: snapshot error", e);
            }
            if (pendingOpen) {
                pendingOpen = false;
                openHud();
            }
        }

        function onEvent(line) {
            if (!line)
                return;
            try {
                const ev = JSON.parse(line);
                if (ev.event === "windows") {
                    windowsAll = ev.data;
                    trackFocus();
                } else if (ev.event === "outputs") {
                    outputsAll = ev.data;
                    trackFocus();
                }
            } catch (e) {
                // línea incompleta o evento no JSON: se ignora
            }
        }

        // ---------------------------------------------------------------- input
        Process {
            id: subscription
            command: ["umbriel", "subscribe", "windows,outputs"]
            running: true
            stdout: SplitParser {
                onRead: (line) => hud.onEvent(line)
            }
            onExited: restartTimer.restart()
        }

        Timer {
            id: restartTimer
            interval: 2000
            repeat: false
            onTriggered: subscription.running = true
        }

        // Respaldo si la suscripción aún no ha entregado nada.
        Process {
            id: snapshotProcess
            command: ["sh", "-c", "umbriel outputs --json; echo '---'; umbriel windows --json"]
            stdout: StdioCollector {
                id: snapshotOut
                onStreamFinished: hud.onSnapshot(snapshotOut.text)
            }
        }

        Process {
            id: captureProcess
            onExited: (code, status) => {
                if (!hud.captureWanted)
                    return;
                hud.captureWanted = false;
                if (code === 0)
                    hud.shotPath = "file://" + hud.pendingShot;
                else
                    hud.shotPath = "";
                hud.showHud();
            }
        }

        Process {
            id: focusProcess
            command: ["sh", "-c", "true"]
        }

        Timer {
            id: idleTimer
            interval: 850
            repeat: false
            onTriggered: hud.commit()
        }

        // ------------------------------------------------------------- backdrop
        Image {
            id: backdrop
            anchors.fill: parent
            source: hud.shotPath
            fillMode: Image.PreserveAspectCrop
            sourceClipRect: hud.backdropClip
            visible: false
        }

        MultiEffect {
            anchors.fill: parent
            source: backdrop
            blurEnabled: true
            blur: 0.8
            brightness: -0.48
            saturation: -0.3
            opacity: hud.reveal ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: 180 } }
        }

        Rectangle {
            anchors.fill: parent
            color: "#B805060A"
        }

        Canvas {
            anchors.fill: parent
            opacity: 0.05
            onPaint: {
                const ctx = getContext("2d");
                ctx.clearRect(0, 0, width, height);
                ctx.strokeStyle = "#7CFFB2";
                ctx.lineWidth = 1;
                const step = 80;
                for (let x = 0; x < width; x += step) {
                    ctx.beginPath();
                    ctx.moveTo(x, 0);
                    ctx.lineTo(x, height);
                    ctx.stroke();
                }
                for (let y = 0; y < height; y += step) {
                    ctx.beginPath();
                    ctx.moveTo(0, y);
                    ctx.lineTo(width, y);
                    ctx.stroke();
                }
            }
        }

        // línea de barrido del HUD
        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            height: 2
            color: hud.accent
            opacity: hud.reveal ? 0.20 : 0
            visible: opacity > 0

            SequentialAnimation on y {
                loops: Animation.Infinite
                NumberAnimation {
                    from: hud.height * 0.14
                    to: hud.height * 0.86
                    duration: 3400
                    easing.type: Easing.InOutSine
                }
                PauseAnimation { duration: 1100 }
            }
        }

        // --------------------------------------------------------------- chrome
        Item {
            id: chrome
            anchors.fill: parent
            opacity: hud.reveal ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: 220 } }

            Repeater {
                model: [
                    { x: 34, y: 30, sx: 1, sy: 1 },
                    { x: -34, y: 30, sx: -1, sy: 1 },
                    { x: 34, y: -30, sx: 1, sy: -1 },
                    { x: -34, y: -30, sx: -1, sy: -1 }
                ]
                delegate: Item {
                    required property var modelData
                    x: modelData.x > 0 ? modelData.x : parent.width + modelData.x
                    y: modelData.y > 0 ? modelData.y : parent.height + modelData.y
                    width: 46
                    height: 46

                    Rectangle {
                        x: modelData.sx > 0 ? 0 : parent.width - width
                        width: 46
                        height: 2
                        color: hud.accent
                        opacity: 0.8
                    }
                    Rectangle {
                        x: modelData.sx > 0 ? 0 : parent.width - width
                        y: modelData.sy > 0 ? 0 : parent.height - height
                        width: 2
                        height: 46
                        color: hud.accent
                        opacity: 0.8
                    }
                }
            }

            Column {
                x: 64
                y: 62
                spacing: 6

                Row {
                    spacing: 12
                    Rectangle {
                        width: 10
                        height: 10
                        color: hud.accent
                        anchors.verticalCenter: parent.verticalCenter
                    }
                    Text {
                        text: "RAKIA"
                        color: "#E8F6EC"
                        font.pixelSize: 22
                        font.bold: true
                        font.letterSpacing: 6
                    }
                    Text {
                        text: "// WINDOW MATRIX"
                        color: hud.accent
                        font.pixelSize: 12
                        font.family: "monospace"
                        font.letterSpacing: 3
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }

                Rectangle {
                    width: 340
                    height: 1
                    color: hud.accent
                    opacity: 0.45
                }

                Text {
                    text: "SESIÓN  ·  " + hud.list.length + " OBJETIVOS  ·  "
                        + hud.outputsAll.length + " MONITORES"
                    color: "#8B95A7"
                    font.pixelSize: 11
                    font.family: "monospace"
                    font.letterSpacing: 2
                }
            }

            Column {
                anchors.right: parent.right
                anchors.rightMargin: 64
                y: 62
                spacing: 4

                Text {
                    anchors.right: parent.right
                    text: Qt.formatTime(new Date(), "HH:mm")
                    color: "#E8F6EC"
                    font.pixelSize: 30
                    font.family: "monospace"
                    font.letterSpacing: 2
                }
                Text {
                    anchors.right: parent.right
                    text: "SISTEMA ESTABLE"
                    color: hud.accentSoft
                    font.pixelSize: 10
                    font.family: "monospace"
                    font.letterSpacing: 3
                }
            }

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                y: parent.height * 0.18
                text: (hud.selected + 1).toString().padStart(2, "0")
                color: hud.accent
                opacity: 0.11
                font.pixelSize: Math.round(parent.height * 0.55)
                font.bold: true
                font.family: "monospace"
            }
        }

        // clic fuera de las tarjetas: cancelar; rueda: navegar
        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.LeftButton
            onClicked: hud.closeHud()
            onWheel: (wheel) => hud.advance(wheel.angleDelta.y > 0 ? -1 : 1)
        }

        // -------------------------------------------------------------- carousel
        Item {
            id: stage
            anchors.fill: parent
            opacity: hud.reveal ? 1 : 0
            scale: hud.reveal ? 1 : 0.94
            Behavior on opacity { NumberAnimation { duration: 220 } }
            Behavior on scale { NumberAnimation { duration: 300; easing.type: Easing.OutCubic } }

            Repeater {
                model: hud.list

                delegate: Item {
                    id: card
                    required property var modelData
                    required property int index

                    readonly property int count: hud.list.length
                    readonly property int rawOffset: index - hud.selected
                    readonly property int offset: {
                        if (count <= 1)
                            return 0;
                        let o = rawOffset;
                        const half = Math.ceil(count / 2);
                        if (o >= half)
                            o -= count;
                        else if (o < -half)
                            o += count;
                        return o;
                    }
                    readonly property bool active: offset === 0
                    readonly property real dist: Math.abs(offset)
                    readonly property var clip: hud.shotPath !== "" ? hud.previewRect(modelData) : null
                    readonly property bool hasPreview: clip !== null
                    readonly property string iconSource: {
                        const id = modelData.app_id;
                        const parts = [id, id.split(".").pop()];
                        for (const candidate of parts) {
                            const path = Quickshell.iconPath(candidate, true);
                            if (path)
                                return path;
                        }
                        return "";
                    }

                    width: hud.cardW
                    height: hud.cardH

                    x: hud.width / 2 - hud.cardW / 2 + offset * hud.cardW * 0.56
                    y: hud.height * 0.52 - hud.cardH / 2
                    z: 100 - dist
                    visible: dist <= 2
                    opacity: dist === 0 ? 1 : Math.max(0, 0.62 - (dist - 1) * 0.24)
                    scale: dist === 0 ? 1 : Math.max(0.62, 0.82 - (dist - 1) * 0.08)

                    transform: Rotation {
                        origin.x: card.width / 2
                        origin.y: card.height / 2
                        axis { x: 0; y: 1; z: 0 }
                        angle: -card.offset * 34
                    }

                    Behavior on x { NumberAnimation { duration: 260; easing.type: Easing.OutCubic } }
                    Behavior on scale { NumberAnimation { duration: 260; easing.type: Easing.OutCubic } }
                    Behavior on opacity { NumberAnimation { duration: 200 } }

                    Rectangle {
                        id: auraSource
                        anchors.centerIn: parent
                        width: parent.width * 1.10
                        height: parent.height * 1.16
                        radius: 40
                        color: hud.accent
                        visible: false
                    }

                    MultiEffect {
                        anchors.fill: auraSource
                        source: auraSource
                        blurEnabled: true
                        blur: 0.9
                        opacity: card.active ? 0.16 : 0
                        visible: opacity > 0
                        Behavior on opacity { NumberAnimation { duration: 200 } }
                    }

                    Rectangle {
                        id: frame
                        anchors.fill: parent
                        radius: 18
                        color: "#0A0F14"
                        border.width: card.active ? 2 : 1
                        border.color: card.active
                            ? hud.accent
                            : Qt.rgba(0.49, 1, 0.70, 0.22)

                        Rectangle {
                            anchors.fill: parent
                            anchors.margins: 1
                            radius: 17
                            color: "transparent"
                            border.width: 1
                            border.color: Qt.rgba(1, 1, 1, card.active ? 0.10 : 0.04)
                        }

                        Item {
                            id: screen
                            anchors.fill: parent
                            anchors.margins: 10
                            clip: true

                            Rectangle {
                                anchors.fill: parent
                                gradient: Gradient {
                                    GradientStop { position: 0.0; color: "#101820" }
                                    GradientStop { position: 1.0; color: "#070A0E" }
                                }
                            }

                            Image {
                                anchors.fill: parent
                                source: card.hasPreview ? hud.shotPath : ""
                                fillMode: Image.PreserveAspectCrop
                                asynchronous: true
                                smooth: true
                                mipmap: true
                                sourceClipRect: card.clip ? card.clip : Qt.rect(0, 0, 0, 0)
                            }

                            // Placeholder: ventana en otro workspace / minimizada.
                            Item {
                                anchors.fill: parent
                                visible: !card.hasPreview

                                Column {
                                    anchors.centerIn: parent
                                    spacing: 12

                                    Image {
                                        width: 88
                                        height: 88
                                        anchors.horizontalCenter: parent.horizontalCenter
                                        source: card.iconSource
                                        sourceSize.width: 176
                                        sourceSize.height: 176
                                        opacity: 0.42
                                        visible: status === Image.Ready
                                        smooth: true
                                    }

                                    Text {
                                        anchors.horizontalCenter: parent.horizontalCenter
                                        text: "EN OTRO WORKSPACE"
                                        color: hud.accentSoft
                                        font.pixelSize: 10
                                        font.family: "monospace"
                                        font.letterSpacing: 3
                                    }
                                    Text {
                                        anchors.horizontalCenter: parent.horizontalCenter
                                        text: card.modelData.output + "  ·  WS "
                                            + card.modelData.workspace.split(":").pop()
                                        color: "#7C8A99"
                                        font.pixelSize: 9
                                        font.family: "monospace"
                                    }
                                }
                            }

                            Image {
                                anchors.centerIn: parent
                                width: parent.width * 0.32
                                height: width
                                source: card.iconSource
                                sourceSize.width: 256
                                sourceSize.height: 256
                                opacity: card.active ? 0.08 : 0.05
                                visible: card.hasPreview && status === Image.Ready
                                smooth: true
                            }

                            Rectangle {
                                anchors.fill: parent
                                color: hud.accent
                                opacity: card.active ? 0.05 : 0.10
                            }

                            Rectangle {
                                anchors.fill: parent
                                color: hud.ink
                                opacity: card.active ? 0 : 0.45
                                Behavior on opacity { NumberAnimation { duration: 200 } }
                            }

                            Rectangle {
                                anchors.left: parent.left
                                anchors.right: parent.right
                                anchors.bottom: parent.bottom
                                height: parent.height * 0.42
                                gradient: Gradient {
                                    GradientStop { position: 0.0; color: "transparent" }
                                    GradientStop { position: 1.0; color: Qt.rgba(0.02, 0.03, 0.05, 0.92) }
                                }
                            }

                            Row {
                                anchors.left: parent.left
                                anchors.bottom: parent.bottom
                                anchors.margins: 16
                                spacing: 12

                                Item {
                                    width: 34
                                    height: 34
                                    anchors.verticalCenter: parent.verticalCenter

                                    Image {
                                        id: appIcon
                                        anchors.fill: parent
                                        source: card.iconSource
                                        sourceSize.width: 68
                                        sourceSize.height: 68
                                        smooth: true
                                        visible: status === Image.Ready
                                    }

                                    Rectangle {
                                        anchors.fill: parent
                                        visible: !appIcon.visible
                                        radius: 7
                                        color: Qt.rgba(0.49, 1, 0.70, 0.10)
                                        border.width: 1
                                        border.color: Qt.rgba(0.49, 1, 0.70, 0.35)

                                        Text {
                                            anchors.centerIn: parent
                                            text: {
                                                const p = card.modelData.app_id.split(".");
                                                return p[p.length - 1].substring(0, 2).toUpperCase();
                                            }
                                            color: hud.accent
                                            font.pixelSize: 12
                                            font.bold: true
                                            font.family: "monospace"
                                        }
                                    }
                                }

                                Column {
                                    spacing: 2
                                    anchors.verticalCenter: parent.verticalCenter

                                    Text {
                                        width: screen.width - 130
                                        text: card.modelData.title !== ""
                                            ? card.modelData.title
                                            : card.modelData.app_id
                                        color: "#ECF7F1"
                                        font.pixelSize: 16
                                        font.bold: true
                                        elide: Text.ElideRight
                                    }
                                    Text {
                                        text: card.modelData.app_id
                                        color: hud.accentSoft
                                        font.pixelSize: 10
                                        font.family: "monospace"
                                        elide: Text.ElideRight
                                    }
                                }
                            }

                            Rectangle {
                                anchors.right: parent.right
                                anchors.top: parent.top
                                anchors.margins: 12
                                width: badge.width + 16
                                height: 22
                                radius: 4
                                color: Qt.rgba(0.02, 0.03, 0.05, 0.8)
                                border.width: 1
                                border.color: Qt.rgba(0.49, 1, 0.70, 0.35)

                                Text {
                                    id: badge
                                    anchors.centerIn: parent
                                    text: card.modelData.output + " · " + card.modelData.workspace.split(":").pop()
                                    color: hud.accent
                                    font.pixelSize: 10
                                    font.family: "monospace"
                                }
                            }

                            MouseArea {
                                anchors.fill: parent
                                acceptedButtons: Qt.LeftButton
                                onClicked: {
                                    hud.selected = card.index;
                                    hud.commit();
                                }
                            }
                        }

                        Item {
                            anchors.fill: parent
                            anchors.margins: -10
                            visible: card.active

                            Rectangle { x: 0; y: 0; width: 18; height: 2; color: hud.accent }
                            Rectangle { x: 0; y: 0; width: 2; height: 18; color: hud.accent }
                            Rectangle { x: parent.width - width; y: 0; width: 18; height: 2; color: hud.accent }
                            Rectangle { x: parent.width - width; y: 0; width: 2; height: 18; color: hud.accent }
                            Rectangle { x: 0; y: parent.height - height; width: 18; height: 2; color: hud.accent }
                            Rectangle { x: 0; y: parent.height - height; width: 2; height: 18; color: hud.accent }
                            Rectangle { x: parent.width - width; y: parent.height - height; width: 18; height: 2; color: hud.accent }
                            Rectangle { x: parent.width - width; y: parent.height - height; width: 2; height: 18; color: hud.accent }

                            Rectangle {
                                width: parent.width
                                height: 2
                                color: hud.accent
                                opacity: 0.7

                                SequentialAnimation on y {
                                    loops: Animation.Infinite
                                    NumberAnimation {
                                        from: 0
                                        to: screen.height
                                        duration: 2600
                                        easing.type: Easing.InOutSine
                                    }
                                    PauseAnimation { duration: 900 }
                                }
                            }
                        }
                    }
                }
            }
        }

        // ---------------------------------------------------------------- footer
        Item {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: 92
            opacity: hud.reveal ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: 220 } }

            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                height: 1
                color: hud.accent
                opacity: 0.22
            }

            Row {
                anchors.left: parent.left
                anchors.leftMargin: 64
                anchors.verticalCenter: parent.verticalCenter
                spacing: 26

                Repeater {
                    model: [
                        { key: "TAB / ← →", label: "NAVEGAR" },
                        { key: "ENTER", label: "IR" },
                        { key: "ESC", label: "SALIR" }
                    ]
                    delegate: Row {
                        required property var modelData
                        spacing: 8

                        Rectangle {
                            width: keyLabel.implicitWidth + 16
                            height: 24
                            radius: 4
                            color: "transparent"
                            border.width: 1
                            border.color: Qt.rgba(0.49, 1, 0.70, 0.4)

                            Text {
                                id: keyLabel
                                anchors.centerIn: parent
                                text: modelData.key
                                color: hud.accent
                                font.pixelSize: 10
                                font.family: "monospace"
                                font.letterSpacing: 1
                            }
                        }

                        Text {
                            text: modelData.label
                            color: "#7C8A99"
                            font.pixelSize: 10
                            font.family: "monospace"
                            font.letterSpacing: 2
                            anchors.verticalCenter: parent.verticalCenter
                        }
                    }
                }
            }

            Row {
                anchors.right: parent.right
                anchors.rightMargin: 64
                anchors.verticalCenter: parent.verticalCenter
                spacing: 16

                Text {
                    text: hud.list.length > 0
                        ? (hud.selected + 1).toString().padStart(2, "0")
                        : "00"
                    color: hud.accent
                    font.pixelSize: 26
                    font.family: "monospace"
                    font.bold: true
                }
                Text {
                    text: "/ " + hud.list.length.toString().padStart(2, "0")
                    color: "#7C8A99"
                    font.pixelSize: 14
                    font.family: "monospace"
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.verticalCenterOffset: 4
                }
            }
        }

        // ------------------------------------------------------------- keyboard
        Item {
            id: keyCatcher
            anchors.fill: parent
            focus: true

            Keys.onPressed: (event) => {
                if (!hud.visible) {
                    event.accepted = false;
                    return;
                }

                switch (event.key) {
                case Qt.Key_Tab:
                case Qt.Key_Right:
                case Qt.Key_Down:
                case Qt.Key_J:
                    hud.advance((event.modifiers & Qt.ShiftModifier) ? -1 : 1);
                    event.accepted = true;
                    break;
                case Qt.Key_Left:
                case Qt.Key_Up:
                case Qt.Key_K:
                    hud.advance(-1);
                    event.accepted = true;
                    break;
                case Qt.Key_Return:
                case Qt.Key_Enter:
                case Qt.Key_Space:
                    hud.commit();
                    event.accepted = true;
                    break;
                case Qt.Key_Escape:
                    hud.closeHud();
                    event.accepted = true;
                    break;
                }
            }

            Keys.onReleased: (event) => {
                if (event.key === Qt.Key_Alt && hud.visible) {
                    hud.commit();
                    event.accepted = true;
                }
            }
        }

        Component.onCompleted: {
            keyCatcher.forceActiveFocus();
        }

        onVisibleChanged: {
            if (visible)
                Qt.callLater(() => keyCatcher.forceActiveFocus());
        }
    }

    // -------------------------------------------------------------------- IPC
    IpcHandler {
        target: "hud"

        function ping(): bool {
            return true;
        }

        function open(): void {
            hud.openHud(1);
        }

        function next(): void {
            if (hud.visible)
                hud.advance(1);
            else
                hud.openHud(1);
        }

        function prev(): void {
            if (hud.visible)
                hud.advance(-1);
            else
                hud.openHud(-1);
        }

        function commit(): void {
            if (hud.visible)
                hud.commit();
        }

        function cancel(): void {
            if (hud.visible)
                hud.closeHud();
        }
    }

    Connections {
        target: hud
        function onCommitRequested(windowId: string) {
            const w = hud.windowById(windowId);
            if (!w)
                return;
            const workspace = w.workspace.split(":").pop();
            focusProcess.command = [
                "sh", "-c",
                "umbriel msg workspace-switch " + hud.shq(workspace)
                    + " --output " + hud.shq(w.output)
                    + " && umbriel msg window-focus " + hud.shq(w.id)
            ];
            focusProcess.running = true;
        }
    }
}
