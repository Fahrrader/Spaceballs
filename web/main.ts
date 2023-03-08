declare global {
    interface Window { playArea: [number, number] | null; }
}


export function run(): void {
    for (let loader of document.getElementsByClassName("loader")) {
        loader.remove()
    }
    let c = document.getElementsByTagName("canvas")[0];

    if (detectMob()) {
        setupControls(c);
        for (let panel of document.getElementsByClassName("controls_panel"))
            if (panel instanceof HTMLElement)
                panel.style.visibility = "visible";
    }

    recalculateCanvasSize(c);
    c.focus();
    onresize = () => recalculateCanvasSize(c);
}

export function detectWindowResize(): boolean {
    return window.playArea != null;
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

function detectMob(): boolean {
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

    for (let key of document.getElementsByClassName("key_w"))
        setupOneButton(key, 'w', 'KeyW');
    for (let key of document.getElementsByClassName("key_a"))
        setupOneButton(key, 'a', 'KeyA');
    for (let key of document.getElementsByClassName("key_s"))
        setupOneButton(key, 's', 'KeyS');
    for (let key of document.getElementsByClassName("key_d"))
        setupOneButton(key, 'd', 'KeyD');

    for (let key of document.getElementsByClassName("key_space"))
        setupOneButton(key, ' ', 'Space');
}

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
