export async function populateCollectionSelectAsync(model) {
    await populateSelectAsync("collection",model?.collection_id?.toString());
}

export async function populateParentCollectionSelectAsync(model) {
    await populateSelectAsync("parent-collection", model?.parent_id?.toString());
}

async function populateSelectAsync(selectName, defaultkey) {
    await loadCollectionsAsync().then(options => {
        const select = document.getElementById(selectName);
        select.options.length = 1;
        let i = 1;
        for (const key in options) {
            const selected = defaultkey === key;
            select.options[i] = new Option(options[key], key, undefined, selected);
            i++;
        }

    })
}

async function loadCollectionsAsync() {
    return await fetch("/collections", {
        method: "Get",
    }).then(res=>res.json());
}

