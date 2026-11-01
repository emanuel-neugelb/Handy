// NGLB: neutral placeholder replacing upstream's brand mascot.
const HandyMascot = ({ className = "" }: { className?: string }) => {
  return (
    <svg
      viewBox="0 0 151 165"
      className={className}
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      aria-hidden="true"
    >
      <circle
        cx="75"
        cy="85"
        r="60"
        className="mascot-line"
        stroke="currentColor"
        strokeWidth="6"
      />
      <rect x="62" y="48" width="26" height="48" rx="13" fill="currentColor" />
      <path
        d="M50 82c0 14 11 26 25 26s25-12 25-26M75 108v16"
        stroke="currentColor"
        strokeWidth="6"
        strokeLinecap="round"
      />
    </svg>
  );
};

export default HandyMascot;
