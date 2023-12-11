import { makeOption } from "./components.js"

/** @typedef {{browsers: {name: string, label: string}[]}} BrowserList  */

const browserSelect = document.querySelector("#browserSelect")

/** @param {BrowserList} browserList */
function loadBrowsers(browserList) {
    browserList.browsers.forEach(browser => {
        browserSelect.appendChild(makeOption(browser))
    })
}

window.addEventListener("DOMContentLoaded", async () => {
    await fetch("execute$python ./python/NavigatorList.py", { method: "GET" })
        .then(res => res.json())
        .then(data => loadBrowsers(data))
})