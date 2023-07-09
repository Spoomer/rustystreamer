const videoContainer = document.getElementById("videoContainer");
const videoplayer = document.getElementById("videoplayer");

const videoControls = document.getElementById("video-controls");
const playpause = document.getElementById("playpause");
const mute = document.getElementById("mute");
const volinc = document.getElementById("volinc");
const voldec = document.getElementById("voldec");
const volumeProgress = document.getElementById("volume-progress");
const volumeProgressBar = document.getElementById("volume-progress-bar");
const fullscreen = document.getElementById("fs");
const       progressBar = document.getElementById('progress-bar');
const progress = document.getElementById('progress');
const circle = document.getElementById('circle');

//timestamp
videoplayer.onloadedmetadata = function () {
    getTimestamp()
    // videoProgress.setAttribute("max", videoplayer.duration);
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
// let mouseOverControls = false;
// videoplayer.onmouseover = () => {
//     if (!mouseOverControls) {
//         videoControls.setAttribute("data-state", "visible")
//     }
// }
// videoplayer.onmouseout = () => {
//     if (!mouseOverControls) {
//         videoControls.setAttribute("data-state", "hidden")
//     }
// }
// videoControls.onmouseover = () => {
//     mouseOverControls = true;
// }
// videoControls.onmouseout = () => {
//     mouseOverControls = false;
// }
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
    // if (!videoProgress.getAttribute("max")) {
    //     videoProgress.setAttribute("max", videoplayer.duration);
    // }
    // videoProgress.value = videoplayer.currentTime;
    // videoProgressBar.style.width = `${Math.floor((videoplayer.currentTime * 100) / videoplayer.duration)}%`;

    const progressPercent = (videoplayer.currentTime / videoplayer.duration) * 100;
    progress.style.width = `${progressPercent}%`;
    circle.style.left = `${progressPercent}%`;
}

// videoProgress.onclick = (e) => {
//     const pos =
//         (e.pageX - videoProgress.offsetLeft - videoProgress.offsetParent.offsetLeft) /
//         videoProgress.offsetWidth;
//     videoplayer.currentTime = pos * videoplayer.duration;
// }
// videoProgress.onmouseover = (e) => {
//     const pos =
//         (e.pageX - videoProgress.offsetLeft - videoProgress.offsetParent.offsetLeft) /
//         videoProgress.offsetWidth;
//     const currentTime = pos * videoplayer.duration;
//     console.log(currentTime);
// }

circle.addEventListener('mousedown', () => {
    videoplayer.pause();
    document.addEventListener('mousemove', moveCircle);
});

document.addEventListener('mouseup', () => {
    document.removeEventListener('mousemove', moveCircle);
    videoplayer.play();
});

function moveCircle(e) {
    console.log(e.pageX);
    const progressBarWidth = progressBar.offsetWidth;
    console.log(progress.clientWidth);
    const progressRect = progress.getBoundingClientRect();
    const clickX = (e.pageX - progressRect.x) / progress.offsetWidth;
    // clickX = event.clientX - circle.offsetWidth / 2;
    const progressPercent = (clickX / progressBarWidth) * 100;

    if (progressPercent >= 0 && progressPercent <= 100) {
        progress.style.width = `${progressPercent}%`;
        circle.style.left = `${progressPercent}%`;
        videoplayer.currentTime = (progressPercent / 100) * videoplayer.duration;
    }
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
