function postCollection() {
    const collectionTitle = document.getElementById("collection-title").value;
    const parentCollection = document.getElementById("parent-collection").value;
    fetch("/collection", {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        keepalive: true,
        body: JSON.stringify(
            {
                collection_id: 0,
                title : collectionTitle,
                parent_id: parentCollection === "None" ? null  : parseInt(parentCollection)
            })
    }).then(x => x.ok);
    populateCollectionSelect()
    document.getElementById("collection-dialog").close();
}

function populateCollectionSelect() {
    populateSelect("collection");
}

function populateParentCollectionSelect() {
    populateSelect("parent-collection");
}

function populateSelect(selectName) {
    loadCollections().then(options => {
        const select = document.getElementById(selectName);
        select.options.length = 1;
        let i = 1;
        for (const key in options) {
            select.options[i] = new Option(options[key], key);
            i++;
        }

    })
}

function loadCollections() {
    return fetch("/collections", {
        method: "Get",
    }).then(res => res.json()).then(json => {
        return json;
    });
}

document.getElementById("show-collection-dialog").addEventListener("click", () => {
    populateParentCollectionSelect();
    document.getElementById("collection-dialog").showModal();
});

window.addEventListener("load", () => {
    populateCollectionSelect();
})