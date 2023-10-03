import {populateParentCollectionSelectAsync} from "./populate-collection-select.js";
const id = location.pathname.split('/')[3];
export const model = await fetch(`/collection-entry/${id}`).then(res=> res.json());
document.getElementById("collection-id").value = model.collection_id;
document.getElementById("collection-title").value = model.title;
await populateParentCollectionSelectAsync(model);

const form = document.getElementById("edit-collection-form");
form.addEventListener("submit", ()=>{
    if(form.parent_id.value === ""){
        form.parent_id.disabled = true;
    }     
})
document.getElementById("deleteButton").addEventListener("click",async ()=>{
    const res = await fetch(`/collection-entry/${id}`,{
        method: "DELETE",
    });
    if(res.ok){
        document.location.replace(form.action.split('?')[1].split('=')[1]);
    }
});
