const needed_translations = [
    "Não existem navegadores compatíveis instalados no sistema!",
    "Navegador padrão<br>dos WebApps:"
]

/** @returns {Promise<{[key: string]: string}>} */
async function getTranslations() {
    return await fetch(`/execute$python ./python/get_translations.py '${JSON.stringify(needed_translations, null, 0)}'`, { method: "GET" })
        .then(res => res.json())
        .then(data => {
            console.log(data)
            return data
        })
}

export const translations = await getTranslations()