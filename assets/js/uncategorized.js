import {populateCollectionSelectAsync, populateParentCollectionSelectAsync} from "./populate-collection-select.js";

const id = location.pathname.split('/')[3];
if(id !== undefined){
    const model = await fetch(`/video-entry/${id}`).then(res=> res.json());
    document.getElementById("title").value = model.title;
    document.getElementById("video-id").value = model.video_id;
    await populateCollectionSelectAsync(model);
}
else{
    await populateCollectionSelectAsync();
}
document.getElementById("show-collection-dialog").addEventListener("click", async () => {
    await populateParentCollectionSelectAsync();
    document.getElementById("collection-dialog").showModal();
});
document.getElementById("postCollectionButton").addEventListener("click",postCollectionAsync);
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

