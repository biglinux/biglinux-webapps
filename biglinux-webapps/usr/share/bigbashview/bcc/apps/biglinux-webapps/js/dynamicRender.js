import { makeOption } from "./components.js"

/** @typedef {{browsers: {name: string, label: string}[]}} BrowserList  */

const browserSelect = document.querySelector("#browserSelect")

/** @param {BrowserList} browserList */
function loadBrowsers(browserList) {
    browserList.browsers.forEach(browser => {
        browserSelect.appendChild(makeOption({
            label: browser.label,
            value: browser.name
        }))
    })

    const firstOption = $("#browserSelect option").first()
    const firstValue = firstOption.val()
    console.log("First-Browser-Combobox: " + firstValue)
    wrapper_browser(firstValue)

    if (browserList.browsers.length === 0) {
        $("#change-browser").innerHTML = `
        <div class="pop-up__subtitle">
          Não existem navegadores compatíveis instalados no sistema!
        </div>
        <div class="content-button-wrapper">
          <button class="content-button status-button2 close">"Fechar"</button>
        </div>`
    }


}

window.addEventListener("DOMContentLoaded", async () => {
    await fetch("execute$python ./python/NavigatorList.py", { method: "GET" })
        .then(res => res.json())
        .then(data => {
            console.log("Loaded Browsers:")
            console.log(data)
            loadBrowsers(data)
        })
})