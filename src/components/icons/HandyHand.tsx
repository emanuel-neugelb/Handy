// NGLB: neutral microphone glyph replacing upstream's brand artwork.
const HandyHand = ({
  width,
  height,
}: {
  width?: number | string;
  height?: number | string;
  className?: string;
}) => (
  <svg
    width={width || 126}
    height={height || 135}
    viewBox="0 0 126 135"
    className="fill-text stroke-text"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect x="45" y="8" width="36" height="70" rx="18" />
    <path
      d="M24 62c0 22 17 40 39 40s39-18 39-40M63 102v24M44 126h38"
      fill="none"
      strokeWidth="8"
      strokeLinecap="round"
    />
  </svg>
);

export default HandyHand;
