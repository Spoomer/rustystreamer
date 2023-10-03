let editLinks = document.getElementsByClassName("edit-link");
for (const link of editLinks) {
  link.href = link.href + `?return_url=${encodeURI(location.pathname)}`
}
