const videoplayer = document.getElementById("Videoplayer");
videoplayer.onloadedmetadata = function () {
    getTimestamp()
};

let intervalSet = false;
videoplayer.onplay =
    function () {
        if (!intervalSet) {
            setInterval(function () { sendTimestamp(); }, 5000);
            intervalSet = true;
        }
    };

document.onvisibilitychange = () => {
    if (document.visibilityState === "hidden") {
        sendTimestamp();
    }
};

function sendTimestamp() {
    fetch("/update-video-timestamp", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        keepalive: true,
        body: JSON.stringify({ id: videoplayer.getAttribute("data-videoId"), timestamp: Math.floor(videoplayer.currentTime) })
    }).then(x => x.ok);
}

function getTimestamp() {
    fetch(`/video-timestamp/${videoplayer.getAttribute("data-videoId")}`).then(x => x.json()).then(x => document.getElementById("Videoplayer").currentTime = x.timestamp);
}