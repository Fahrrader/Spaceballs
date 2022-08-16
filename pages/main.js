export {detectMob, getSceneFromUrl, setupCanvasSize, setupControls};

function getUrlParam(param) {
    const paramString = window.location.search.slice(1);
    const searchParams = new URLSearchParams(paramString);
    const result = searchParams.get(param);
    return String(result);
}

function getSceneFromUrl() {
    return getUrlParam("scene");
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

    if (wid/hei > ratio)
        wid = ratio * hei;
    else
        hei = wid / ratio;

    c.style.width = wid + "px";
    c.style.height = hei + "px";
};
