import * as nipplejs from 'nipplejs';

const STICK_SIZE_PORTRAIT = 200;
const STICK_SIZE_LANDSCAPE = 100;

declare global {
    interface Window {
        playArea: [number, number] | null;
        joysticks: [nipplejs.JoystickManager, nipplejs.JoystickManager] | null;
        sticksPosition: [number, number];
        pasteBufferMap: Map<number, PasteData>;
    }
}

window.playArea = null;
window.joysticks = null;
window.sticksPosition = [0, 0];
window.pasteBufferMap = new Map();

export function run(): void {
    for (let loader of document.getElementsByClassName("loader")) {
        loader.remove()
    }
    let c = document.getElementsByTagName("canvas")[0];

    if (detectMob()) {
        setupControls(c);
        for (let panel of document.getElementsByClassName("controls_panel"))
            (panel as HTMLElement).style.visibility = "visible";
    }

    recalculateCanvasSize(c);
    c.focus();
    onresize = () => recalculateCanvasSize(c);
}

export function detectWindowResize(): boolean {
    return window.playArea !== null;
}

export function getNewWindowSize(): [number, number] | null {
    const size = window.playArea;
    window.playArea = null;
    return size;
}

export function getSceneFromUrl(): string {
    return getUrlParam("scene");
}

function getUrlParam(param: string): string {
    const paramString = window.location.search.slice(1);
    const searchParams = new URLSearchParams(paramString);
    const result = searchParams.get(param);
    return String(result);
}

class PasteData {
    constructor(
        public paste: string,
        public error: string
    ) { }
}

export function setPasteBuffer(entity_index: number) {
    window.navigator.clipboard.readText()
        .then(paste => window.pasteBufferMap.set(entity_index, new PasteData(paste, "")))
        .catch(err => window.pasteBufferMap.set(entity_index, new PasteData("", err)));
}

export function getPasteBuffer(entity_index: number) {
    const clipboardData = window.pasteBufferMap.get(entity_index);

    if (clipboardData) {
        window.pasteBufferMap.delete(entity_index);
        if (clipboardData.error) throw new Error(clipboardData.error);
        return clipboardData.paste;
    }
}

export function getSticksPosition(): [number, number] {
    return window.sticksPosition;
}

export function detectMob(): boolean {
    const toMatch = [
        /Android/i,
        /webOS/i,
        /iPhone/i,
        /iPad/i,
        /iPod/i,
        /BlackBerry/i,
        /Windows Phone/i
    ];

    return toMatch.some((toMatchItem) => {
        return navigator.userAgent.match(toMatchItem);
    });
}

function setupControls(c: HTMLCanvasElement): void {
    let fakePress = function(key: string, code: string, type: string) {
        c.dispatchEvent(new KeyboardEvent(type, {'key': key, 'code': code}));
    }
    let setupOneButton = function(button: Element, key: string, code: string) {
        if (!(button instanceof HTMLElement))
            return;
        button.onmousedown   = (e) => { fakePress(key, code, 'keydown'); e.preventDefault(); return false; }
        button.onmouseup     = (e) => { fakePress(key, code, 'keyup'); e.preventDefault(); return false; }
        button.ontouchstart  = (e) => { fakePress(key, code, 'keydown'); e.preventDefault(); return false; }
        button.ontouchend    = (e) => { fakePress(key, code, 'keyup'); e.preventDefault(); return false; }
        button.ontouchcancel = (e) => { fakePress(key, code, 'keyup'); e.preventDefault(); return false; }
    }

    for (let key of document.getElementsByClassName("key_f"))
        setupOneButton(key, 'f', 'KeyF');
    for (let key of document.getElementsByClassName("key_c"))
        setupOneButton(key, 'c', 'KeyC');

    for (let key of document.getElementsByClassName("key_space"))
        setupOneButton(key, ' ', 'Space');
    
    recreateJoysticks(isLandscape(screen.orientation || window.orientation));

    // todo dem brokey. displays when starting in mobile simulation, stays on change to desktop; doesn't display when entering mobile simulation.
    if(screen.orientation)
        screen.orientation.addEventListener("change", (e)=>{
            recreateJoysticks(isLandscape(e.target as ScreenOrientation));
        })
    else
        window.addEventListener("orientationchange", (e)=>{
            recreateJoysticks(isLandscape((e.target as Window).orientation));
        })
}

function isLandscape(orientation: ScreenOrientation | number): boolean {
    if (typeof(orientation) === 'number')
        return orientation == -90 || orientation == 90;
    return orientation.type.startsWith("landscape");
}

function recreateJoysticks(isLandscape: boolean): void {
    if (window.joysticks !== null) {
        window.joysticks[0].destroy();
        window.joysticks[1].destroy();
    }

    let size = isLandscape ? STICK_SIZE_LANDSCAPE : STICK_SIZE_PORTRAIT;

    var leftStickManager = nipplejs.create({
        zone: document.getElementById('left_stick') as HTMLElement,
        mode: 'static',
        position: {left: '50%', bottom: '50%'},
        color: 'white',
        lockX: true,
        size: size,
    });
    leftStickManager.on("move", (_event, data) => {
        window.sticksPosition[0] = data.vector.x * (data.distance / size) * 2;
    });
    leftStickManager.on("end", (_event, _data) => {
        window.sticksPosition[0] = 0;
    })

    var rightStickManager = nipplejs.create({
        zone: document.getElementById('right_stick') as HTMLElement,
        mode: 'static',
        position: {left: '50%', bottom: '50%'},
        color: 'white',
        lockY: true,
        size: size,
    });
    rightStickManager.on("move", (_event, data) => {
        window.sticksPosition[1] = data.vector.y * (data.distance / size) * 2;
    });
    rightStickManager.on("end", (_event, _data) => {
        window.sticksPosition[1] = 0;
    })

    window.joysticks = [leftStickManager, rightStickManager];
}

// dem also brokey. deal with later
function recalculateCanvasSize(c: HTMLCanvasElement): void {
    let ratio = c.width / c.height;

    let wid = document.body.clientWidth;
    let hei = document.body.clientHeight;

    if (wid/hei > ratio) {
        wid = ratio * hei;
    } else {
        hei = wid / ratio;
    }

    window.playArea = [wid, hei];
}
