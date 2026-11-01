/** NGLB microphone tray icon (same geometry as nglb/make-tray-icons.py); uses currentColor. */
type TrayIconState = "idle" | "recording" | "transcribing";

const TrayIcon = ({
  state,
  size,
}: {
  state: TrayIconState;
  size: number | string;
}) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 64 64"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect
      x="25"
      y="8"
      width="14"
      height="26"
      rx="7"
      fill={state === "idle" ? "none" : "currentColor"}
      stroke="currentColor"
      strokeWidth="4"
    />
    <g fill="none" stroke="currentColor" strokeWidth="4" strokeLinecap="round">
      <path d="M17 34a15 15 0 0 0 30 0M32 49v9M24 58h16" />
    </g>
    {state === "transcribing" &&
      [46, 52, 58].map((x) => (
        <circle key={x} cx={x} cy="32" r="2" fill="currentColor" />
      ))}
  </svg>
);

export default TrayIcon;
