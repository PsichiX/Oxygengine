function bootload(
    channelName,
    startWithSnapshot,
    startWithFiltersFromPipelines,
    scripts,
    images,
) {
    if ('OxygengineHARD' in window) {
        console.warn('Oxygengine HARD is already running!');
        return;
    }

    let elm = document.createElement('div');
    elm.id = 'oxygengine-hard-root';
    elm.style.position = 'fixed';
    elm.style.margin = 0;
    elm.style.border = 0;
    elm.style.top = 0;
    elm.style.left = 0;
    elm.style.width = '600px';
    elm.style.height = '100%';
    elm.style.minWidth = '250px';
    elm.style.maxWidth = '100%';
    elm.style.zIndex = 100000;
    elm.style.resize = 'horizontal';
    elm.style.overflowX = 'hidden';
    elm.style.overflowY = 'auto';
    document.body.appendChild(elm);

    async function loadScript(url, process) {
        const res = await fetch(url);
        const content = await res.text();
        return await new Promise(resolve => {
            let elm = document.createElement('script');
            elm.type = 'text/javascript';
            elm.textContent = !!process ? process(content) : content;
            document.body.appendChild(elm);
            resolve();
        });
    }

    function loadScriptProcessed(url) {
        return loadScript(url, content => {
            if (!!Babel) {
                return Babel.transform(content, { presets: ['env', 'react'] }).code;
            } else {
                throw 'Babel not loaded!';
            }
        });
    }

    Promise.all([
        loadScript(scripts.react),
        loadScript(scripts.reactDom),
        loadScript(scripts.babel),
    ]).then(() => loadScriptProcessed(scripts.index));

    window.OxygengineHARD = {
        channelName,
        startWithSnapshot,
        startWithFiltersFromPipelines,
        images,
    };
}

const defaultChannelName = 'OxygengineHARD';

const run = document.getElementById('run');

const channelName = document.getElementById('channel-name');
if ('channelName' in localStorage) {
    channelName.value = localStorage.channelName || defaultChannelName;
}
channelName.onchange = event => localStorage.channelName = event.target.value;

const startWithSnapshot = document.getElementById('start-with-snapshot');
if ('startWithSnapshot' in localStorage) {
    startWithSnapshot.checked = localStorage.startWithSnapshot === 'true';
}
startWithSnapshot.onchange = event => localStorage.startWithSnapshot = !!event.target.checked;

const startWithFiltersFromPipelines = document.getElementById('start-with-filters-from-pipelines');
if ('startWithFiltersFromPipelines' in localStorage) {
    startWithFiltersFromPipelines.checked = localStorage.startWithFiltersFromPipelines === 'true';
}
startWithFiltersFromPipelines.onchange = event => localStorage.startWithFiltersFromPipelines = !!event.target.checked;

const reset = document.getElementById('reset');
reset.onclick = () => {
    localStorage.clear();
    channelName.value = defaultChannelName;
    startWithSnapshot.checked = false;
    startWithFiltersFromPipelines.checked = false;
};

const scripts = {
    react: chrome.runtime.getURL('debugger/libs/react.development.js'),
    reactDom: chrome.runtime.getURL('debugger/libs/react-dom.development.js'),
    babel: chrome.runtime.getURL('debugger/libs/babel.min.js'),
    index: chrome.runtime.getURL('debugger/index.jsx'),
};
const images = {
    checkerboard: chrome.runtime.getURL('debugger/checkerboard.png'),
};

run.addEventListener('click', async () => {
    if (channelName.value === '') {
        channelName.value = defaultChannelName;
    }
    let [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    const url = new URL(tab.url);
    if (url.hostname !== 'localhost' && url.hostname !== '127.0.0.1') {
        alert('Oxygengine HARD cannot run on non-localhost pages!');
        return;
    }
    chrome.scripting.executeScript({
        target: { tabId: tab.id },
        world: 'MAIN',
        func: bootload,
        args: [
            channelName.value,
            !!startWithSnapshot.checked,
            !!startWithFiltersFromPipelines.checked,
            scripts,
            images,
        ],
    });
});
