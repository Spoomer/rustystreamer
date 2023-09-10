const videoplayer = document.getElementById("Videoplayer");
videoplayer.onloadedmetadata = function () {
    getTimestamp()
};

let intervalId = null;
videoplayer.onplay =
    function () {
        if (!intervalId) {
            intervalId = setInterval(function () { sendTimestamp(); }, 5000);
        }
    };
videoplayer.onpause = function () { if (intervalId){
    sendTimestamp();
    clearInterval(intervalId);
    intervalId =null;
}};

function sendTimestamp() {
    fetch("/update-video-timestamp", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        keepalive: true,
        body: JSON.stringify({ video_id: parseInt(videoplayer.getAttribute("data-videoId")), timestamp: Math.floor(videoplayer.currentTime) })
    }).then(x => x.ok);
}

function getTimestamp() {
    fetch(`/video-timestamp/${videoplayer.getAttribute("data-videoId")}`).then(x => x.json()).then(x => document.getElementById("Videoplayer").currentTime = x.timestamp);
}