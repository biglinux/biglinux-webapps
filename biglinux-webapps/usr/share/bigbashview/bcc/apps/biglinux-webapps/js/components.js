export function makeOption({ value, label }) {
    const option = document.createElement("option")

    option.value = value
    option.innerText = label

    return option
}

export function makeMenuButton({ icon, label, name }) {
    const button = document.createElement("button")

    console.table({ icon, label, name })

    button.classList = "btn-img status-button btn-img-main"
    button.id = name

    button.innerHTML = `
    <img src="${icon}" width="38"
         height="38" data-bin="${name}"
         title="${label}"
    >`

    return button
}