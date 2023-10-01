const id = location.pathname.split('/')[3];
let model = undefined;

document.getElementById("show-collection-dialog").addEventListener("click", async () => {
    await populateParentCollectionSelectAsync();
    document.getElementById("collection-dialog").showModal();
});

window.addEventListener("load", async () => {
    await loadModelAsync();
    await populateCollectionSelectAsync();
})

async function postCollectionAsync() {
    const collectionTitle = document.getElementById("collection-title").value;
    const parentCollection = document.getElementById("parent-collection").value;
    await fetch("/collection", {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        keepalive: true,
        body: JSON.stringify(
            {
                collection_id: 0,
                title: collectionTitle,
                parent_id: parentCollection === "None" ? null : parseInt(parentCollection)
            })
    }).then(x => x.ok);
    await populateCollectionSelectAsync()
    document.getElementById("collection-dialog").close();
}

async function populateCollectionSelectAsync() {
    await populateSelectAsync("collection");
}

async function populateParentCollectionSelectAsync() {
    await populateSelectAsync("parent-collection");
}


async function loadModelAsync() {
    if (model !== undefined || id === undefined) {
        return;
    }
    const res = await fetch(`/video-entry/${id}`);
    model = await res.json();
    document.getElementById("title").value = model.title;
    document.getElementById("video-id").value = model.video_id;
}

async function populateSelectAsync(selectName) {
    await loadCollectionsAsync().then(options => {
        const select = document.getElementById(selectName);
        select.options.length = 1;
        let i = 1;
        for (const key in options) {
            const selected = model?.collection_id.toString() === key;
            select.options[i] = new Option(options[key], key, undefined, selected);
            i++;
        }

    })
}

async function loadCollectionsAsync() {
    const res = await fetch("/collections", {
        method: "Get",
    });
    return await res.json();
}

