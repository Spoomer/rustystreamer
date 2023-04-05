const videoContainer = document.getElementById("videoContainer");
const videoplayer = document.getElementById("videoplayer");

const videoControls = document.getElementById("video-controls");
const playpause = document.getElementById("playpause");
const mute = document.getElementById("mute");
const volinc = document.getElementById("volinc");
const voldec = document.getElementById("voldec");
const videoProgress = document.getElementById("video-progress");
const videoProgressBar = document.getElementById("video-progress-bar");
const volumeProgress = document.getElementById("volume-progress");
const volumeProgressBar = document.getElementById("volume-progress-bar");
const fullscreen = document.getElementById("fs");

//timestamp
videoplayer.onloadedmetadata = function () {
    getTimestamp()
    videoProgress.setAttribute("max", videoplayer.duration);
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
    fetch(`/video-timestamp/${videoplayer.getAttribute("data-videoId")}`).then(x => x.json()).then(x => videoplayer.currentTime = x.timestamp);
}
//controls
videoplayer.controls = false;
playpause.onclick = () => {
    if (videoplayer.paused || videoplayer.ended) {
        videoplayer.play();
    } else {
        videoplayer.pause();
    }
}

videoplayer.addEventListener(
    "play",
    () => {
        changeButtonState("playpause");
    },
    false
);

videoplayer.addEventListener(
    "pause",
    () => {
        changeButtonState("playpause");
    },
    false
);

adjustVolumeBar();

mute.onclick = () => {
    videoplayer.muted = !videoplayer.muted;
    changeButtonState("mute");
}

volinc.onclick = () => {
    alterVolume(true);
}

voldec.onclick = () => {
    alterVolume(false);
}

function alterVolume(up) {
    checkVolume(up);
}

function checkVolume(up) {
    if (up !== undefined) {
        let currentVolume = Math.round(videoplayer.volume * 10) / 10;
        if (up === true && currentVolume < 1) {
            currentVolume += 0.1;
            videoplayer.volume = currentVolume > 1 ? 1 : currentVolume;
        } else if (up === false && currentVolume > 0) {
            videoplayer.volume -= 0.1;
        }
        // If the volume has been turned off, also set it as muted
        // Note: can only do this with the custom control set as when the 'volumechange' event is raised,
        // there is no way to know if it was via a volume or a mute change
        videoplayer.muted = currentVolume <= 0;
    }
    adjustVolumeBar();
    changeButtonState("mute");
}
function adjustVolumeBar() {
    volumeProgress.value = videoplayer.volume;
    volumeProgressBar.style.width = `${Math.floor(videoplayer.volume * 100)}%`;
}
videoplayer.addEventListener(
    "volumechange",
    () => {
        checkVolume();
    },
    false
);
volumeProgress.onclick = (e) => {
    const pos =
        (e.pageX - volumeProgress.offsetLeft - volumeProgress.offsetParent.offsetLeft) /
        volumeProgress.offsetWidth;
    videoplayer.volume = pos;
}

videoplayer.ontimeupdate = () => {
    if (!videoProgress.getAttribute("max")) {
        videoProgress.setAttribute("max", videoplayer.duration);
    }
    videoProgress.value = videoplayer.currentTime;
    videoProgressBar.style.width = `${Math.floor((videoplayer.currentTime * 100) / videoplayer.duration)}%`;
}

videoProgress.onclick = (e) => {
    const pos =
        (e.pageX - videoProgress.offsetLeft - videoProgress.offsetParent.offsetLeft) /
        videoProgress.offsetWidth;
    videoplayer.currentTime = pos * videoplayer.duration;
}

if (!document?.fullscreenEnabled) {
    fullscreen.style.display = "none";
}
fullscreen.onclick = () => {
    handleFullscreen();
}
function handleFullscreen() {
    if (document.fullscreenElement !== null) {
        // The document is in fullscreen mode
        document.exitFullscreen();
        setFullscreenData(false);
    } else {
        // The document is not in fullscreen mode
        videoplayer.requestFullscreen();
        setFullscreenData(true);
    }
}
function setFullscreenData(state) {
    videoplayer.controls = state;

    videoContainer.setAttribute("data-fullscreen", !!state);
}
document.onfullscreenchange = () => {
    setFullscreenData(!!document.fullscreenElement);
};

videoControls.setAttribute("data-state", "visible");

// progress bar not implemented
const supportsProgress = document.createElement("progress").max !== undefined;
if (!supportsProgress) videoProgress.setAttribute("data-state", "fake");


function changeButtonState(type) {
    if (type === "playpause") {
        // Play/Pause button
        if (videoplayer.paused || videoplayer.ended) {
            playpause.setAttribute("data-state", "play");
        } else {
            playpause.setAttribute("data-state", "pause");
        }
    } else if (type === "mute") {
        // Mute button
        mute.setAttribute("data-state", videoplayer.muted ? "mute" : "unmute");
    }
}
