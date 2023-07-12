export {detectMob, detectWindowResize, getNewWindowSize, getSceneFromUrl, getPasteBuffer, setPasteBuffer, setupCanvasSize, setupControls};

function getUrlParam(param) {
    const paramString = window.location.search.slice(1);
    const searchParams = new URLSearchParams(paramString);
    const result = searchParams.get(param);
    return String(result);
}

function getSceneFromUrl() {
    return getUrlParam("scene");
}

function detectWindowResize() {
    return window.playAreaSide !== undefined;
}

function getNewWindowSize() {
    const size = [window.playAreaSide, window.playAreaSide];
    window.playAreaSide = undefined;
    return size;
}

class PasteData {
    constructor(value, error) {
        this.paste = value;
        this.error = error;
    }
}

function setPasteBuffer(entity_index) {
    if (!window.pasteBufferMap) window.pasteBufferMap = new Map();

    window.navigator.clipboard.readText()
        .then(paste => window.pasteBufferMap.set(entity_index, new PasteData(paste, "")))
        .catch(err => window.pasteBufferMap.set(entity_index, new PasteData("", err)));
}

function getPasteBuffer(entity_index) {
    if (window.pasteBufferMap && window.pasteBufferMap.has(entity_index)) {
        const clipboardData = window.pasteBufferMap.get(entity_index);
        window.pasteBufferMap.delete(entity_index);
        if (clipboardData.error) throw new clipboardData.error;
        return clipboardData.paste
    }
}

function detectMob() {
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

let setupControls = (function(c) {
    let fakePress = function(key, code, type) {
        c.dispatchEvent(new KeyboardEvent(type, {'key': key, 'code': code}));
    }
    let setupOneButton = function(button, key, code) {
        button.onmousedown   = (e) => { fakePress(key, code, 'keydown') }
        button.onmouseup     = (e) => { fakePress(key, code, 'keyup') }
        button.ontouchstart  = (e) => { fakePress(key, code, 'keydown'); return false; }
        button.ontouchend    = (e) => { fakePress(key, code, 'keyup'); return false; }
        button.ontouchcancel = (e) => { fakePress(key, code, 'keyup'); return false; }
    }

    setupOneButton(key_w, 'w', 'KeyW');
    setupOneButton(key_a, 'a', 'KeyA');
    setupOneButton(key_s, 's', 'KeyS');
    setupOneButton(key_d, 'd', 'KeyD');

    setupOneButton(key_space, ' ', 'Space');
});


let setupCanvasSize = (c) => {
    let wid = document.body.clientWidth;
    let hei = document.body.clientHeight;

    let ratio = c.width / c.height;

    if (ratio > 1)
        wid /= ratio;
    else
        hei *= ratio;

    ratio = wid / hei;
    if (ratio > 1) {
        window.playAreaSide = hei;
        c.style.height = '100%';
        c.style.width = 100 / ratio + '%';
    } else {
        window.playAreaSide = wid;
        c.style.width = '100%';
        c.style.height = 100 * ratio + '%';
    }
};
