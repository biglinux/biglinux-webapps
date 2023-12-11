export function makeOption({ value, label }) {
    const option = document.createElement("option")

    option.value = value
    option.innerText = label

    return option
}