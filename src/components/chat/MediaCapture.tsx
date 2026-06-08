import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

export interface MediaAttachment {
  name: string;
  mimeType: string;
  sizeBytes: number;
  dataBase64?: string;
  previewUrl?: string;
}

function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string;
      resolve(result.split(",")[1] ?? "");
    };
    reader.onerror = reject;
    reader.readAsDataURL(blob);
  });
}

export function MediaCapture({
  cameraEnabled,
  micEnabled,
  onAttach,
}: {
  cameraEnabled: boolean;
  micEnabled: boolean;
  onAttach: (a: MediaAttachment) => void;
}) {
  const { t } = useTranslation();
  const [cameraOpen, setCameraOpen] = useState(false);
  const [liveOpen, setLiveOpen] = useState(false);
  const [recording, setRecording] = useState(false);
  const [liveRecording, setLiveRecording] = useState(false);
  const [error, setError] = useState("");
  const [micLevel, setMicLevel] = useState(0);
  const videoRef = useRef<HTMLVideoElement>(null);
  const liveVideoRef = useRef<HTMLVideoElement>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const recorderRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const animRef = useRef<number>(0);

  const stopStream = () => {
    cancelAnimationFrame(animRef.current);
    streamRef.current?.getTracks().forEach((tr) => tr.stop());
    streamRef.current = null;
    analyserRef.current = null;
    setMicLevel(0);
  };

  useEffect(() => () => stopStream(), []);

  useEffect(() => {
    if (liveOpen) bindVideo(liveVideoRef.current);
  }, [liveOpen]);

  const bindVideo = async (video: HTMLVideoElement | null) => {
    if (!video || !streamRef.current) return;
    video.srcObject = streamRef.current;
    await video.play();
  };

  const startMicMeter = (stream: MediaStream) => {
    try {
      const ctx = new AudioContext();
      const src = ctx.createMediaStreamSource(stream);
      const analyser = ctx.createAnalyser();
      analyser.fftSize = 256;
      src.connect(analyser);
      analyserRef.current = analyser;
      const data = new Uint8Array(analyser.frequencyBinCount);
      const tick = () => {
        analyser.getByteFrequencyData(data);
        const avg = data.reduce((a, b) => a + b, 0) / data.length;
        setMicLevel(Math.min(100, Math.round((avg / 128) * 100)));
        animRef.current = requestAnimationFrame(tick);
      };
      tick();
    } catch { /* meter optional */ }
  };

  const openCamera = async () => {
    if (!cameraEnabled) {
      setError(t("chat.enableCamera"));
      return;
    }
    setError("");
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: false });
      streamRef.current = stream;
      setCameraOpen(true);
      await bindVideo(videoRef.current);
    } catch (e) {
      setError(t("chat.cameraError", { err: String(e) }));
    }
  };

  const capturePhoto = (video: HTMLVideoElement | null) => {
    if (!video) return;
    const canvas = document.createElement("canvas");
    canvas.width = video.videoWidth || 640;
    canvas.height = video.videoHeight || 480;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.drawImage(video, 0, 0);
    const dataUrl = canvas.toDataURL("image/jpeg", 0.85);
    const base64 = dataUrl.split(",")[1];
    onAttach({
      name: `camera-${Date.now()}.jpg`,
      mimeType: "image/jpeg",
      sizeBytes: Math.round((base64?.length ?? 0) * 0.75),
      dataBase64: base64,
      previewUrl: dataUrl,
    });
  };

  const closeCamera = () => {
    setCameraOpen(false);
    stopStream();
  };

  const toggleMic = async () => {
    if (!micEnabled) {
      setError(t("chat.enableMic"));
      return;
    }
    if (recording) {
      recorderRef.current?.stop();
      return;
    }
    setError("");
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      streamRef.current = stream;
      startMicMeter(stream);
      const recorder = new MediaRecorder(stream);
      chunksRef.current = [];
      recorder.ondataavailable = (e) => { if (e.data.size) chunksRef.current.push(e.data); };
      recorder.onstop = async () => {
        const blob = new Blob(chunksRef.current, { type: "audio/webm" });
        const base64 = await blobToBase64(blob);
        onAttach({
          name: `voice-${Date.now()}.webm`,
          mimeType: "audio/webm",
          sizeBytes: blob.size,
          dataBase64: base64,
        });
        setRecording(false);
        stopStream();
      };
      recorderRef.current = recorder;
      recorder.start();
      setRecording(true);
    } catch (e) {
      setError(t("chat.micError", { err: String(e) }));
    }
  };

  const openLive = async () => {
    if (!cameraEnabled || !micEnabled) {
      setError(t("chat.enableLive"));
      return;
    }
    setError("");
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
      streamRef.current = stream;
      chunksRef.current = [];
      const recorder = new MediaRecorder(stream);
      recorder.ondataavailable = (e) => { if (e.data.size) chunksRef.current.push(e.data); };
      recorderRef.current = recorder;
      recorder.start(250);
      setLiveRecording(true);
      setLiveOpen(true);
      startMicMeter(stream);
      await bindVideo(liveVideoRef.current);
    } catch (e) {
      setError(t("chat.liveError", { err: String(e) }));
    }
  };

  const sendLive = async () => {
    const video = liveVideoRef.current;
    const recorder = recorderRef.current;
    if (!video) return;

    capturePhoto(video);

    if (recorder && liveRecording) {
      await new Promise<void>((resolve) => {
        recorder.onstop = async () => {
          const blob = new Blob(chunksRef.current, { type: "audio/webm" });
          if (blob.size > 0) {
            const base64 = await blobToBase64(blob);
            onAttach({
              name: `live-audio-${Date.now()}.webm`,
              mimeType: "audio/webm",
              sizeBytes: blob.size,
              dataBase64: base64,
            });
          }
          resolve();
        };
        if (recorder.state !== "inactive") recorder.stop();
        else resolve();
      });
    }

    setLiveOpen(false);
    setLiveRecording(false);
    stopStream();
  };

  const closeLive = () => {
    if (recorderRef.current && recorderRef.current.state !== "inactive") {
      recorderRef.current.stop();
    }
    setLiveOpen(false);
    setLiveRecording(false);
    stopStream();
  };

  return (
    <>
      <TooltipBtn title={t("chat.live")} disabled={!cameraEnabled || !micEnabled} active={liveOpen} onClick={openLive}>📹</TooltipBtn>
      <TooltipBtn title={t("chat.camera")} disabled={!cameraEnabled} active={cameraOpen} onClick={openCamera}>📷</TooltipBtn>
      <TooltipBtn title={recording ? t("chat.stopMic") : t("chat.mic")} disabled={!micEnabled} active={recording} onClick={toggleMic}>
        {recording ? "⏹" : "🎤"}
      </TooltipBtn>
      {error && <span className="media-error">{error}</span>}
      {cameraOpen && (
        <div className="camera-overlay">
          <div className="camera-modal m3-card">
            <video ref={videoRef} className="camera-preview" muted playsInline />
            <div className="camera-actions">
              <button type="button" className="m3-filled-btn" onClick={() => { capturePhoto(videoRef.current); closeCamera(); }}>{t("chat.capture")}</button>
              <button type="button" className="m3-outlined-btn" onClick={closeCamera}>{t("chat.cancel")}</button>
            </div>
          </div>
        </div>
      )}
      {liveOpen && (
        <div className="camera-overlay">
          <div className="camera-modal live-modal m3-card">
            <p className="live-status">{t("chat.liveHint")}</p>
            <video ref={liveVideoRef} className="live-preview" muted playsInline />
            <div className="live-meter"><div className="live-meter-fill" style={{ width: `${micLevel}%` }} /></div>
            <div className="camera-actions">
              <button type="button" className="m3-filled-btn" onClick={sendLive}>{t("chat.sendLive")}</button>
              <button type="button" className="m3-outlined-btn" onClick={closeLive}>{t("chat.cancel")}</button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}

function TooltipBtn({ children, title, disabled, active, onClick }: {
  children: React.ReactNode; title: string; disabled?: boolean; active?: boolean; onClick: () => void;
}) {
  return (
    <button type="button" className={`composer-media-btn ${active ? "active" : ""}`} title={title} disabled={disabled} onClick={onClick}>
      {children}
    </button>
  );
}
