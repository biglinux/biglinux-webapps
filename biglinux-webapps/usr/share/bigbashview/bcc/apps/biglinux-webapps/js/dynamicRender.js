import { makeMenuButton, makeOption } from "./components.js"
import { translations } from "./translations.js"

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
          ${translations["Não existem navegadores compatíveis instalados no sistema!"]}
        </div>
        <div class="content-button-wrapper">
          <button class="content-button status-button2 close">"Fechar"</button>
        </div>`
    }

    const nativeMenu = document.querySelector(`[data-menu="native"]`)
    const flatpakMenu = document.querySelector(`[data-menu="flatpak"]`)

    const nativeBrowsers = browserList.browsers.filter(browser => browser.native)
    nativeBrowsers.forEach(browser => nativeMenu.appendChild(makeMenuButton(browser)))

    const flatpakBrowsers = browserList.browsers.filter(browser => browser.flatpak)
    flatpakBrowsers.forEach(browser => flatpakMenu.appendChild(makeMenuButton(browser)))

    $(".btn-img").each(function () {
        var img = $(this).children()[0]
        var src = $(img).attr("src")
        var dataBin = $(img).attr("data-bin")
        var title = $(img).attr("title")
        $(this).click(function () {
            var currBin = $("#open-change-browsers").attr("data-bin")
            if (currBin === dataBin) {
                $(".pop-up#change-browser").removeClass("visible")
            } else {
                $(".pop-up#change-browser").removeClass("visible")
                $(".iconBrowser").attr("src", src)
                $("#open-change-browsers").attr("data-bin", dataBin)
                $("#browserIcon").attr("title", title)
                fetch(`/execute$./change_browser.sh ${currBin} ${dataBin}`)
            }
            console.log("Browser-Old: " + currBin, "Browser-New: " + dataBin)
        }).mouseover(function () {
            $("button.btn-img").removeClass("highlight")
        })
    })
}

// window.addEventListener("load", async () => {
//     console.log("Loading Dynamic Rendering")
await fetch("/execute$python ./python/NavigatorList.py", { method: "GET" })
    .then(res => res.json())
    .then(data => {
        console.log("Loaded Browsers:")
        console.log(data)
        loadBrowsers(data)
    })
// })