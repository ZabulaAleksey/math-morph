import type { SVGProps } from "react";

type IconProps = SVGProps<SVGSVGElement>;

const sharedProps: IconProps = {
  "aria-hidden": true,
  className: "cbui-icon",
  fill: "none",
  stroke: "currentColor",
  strokeLinecap: "round",
  strokeLinejoin: "round",
  strokeWidth: 1.8,
  viewBox: "0 0 24 24",
};

export function BrandMark(props: IconProps) {
  return (
    <svg {...sharedProps} {...props} viewBox="0 0 32 32">
      <path d="M4 26V6l12 13L28 6v20" />
      <path d="M4 6l12 20L28 6" opacity=".45" />
    </svg>
  );
}

export function MenuIcon(props: IconProps) {
  return <svg {...sharedProps} {...props}><path d="M4 7h16M4 12h16M4 17h16" /></svg>;
}

export function ArrowIcon(props: IconProps) {
  return <svg {...sharedProps} {...props}><path d="M5 12h14M14 7l5 5-5 5" /></svg>;
}

export function CheckIcon(props: IconProps) {
  return <svg {...sharedProps} {...props}><path d="m5 12 4 4L19 6" /></svg>;
}

export function Icon({ name }: Readonly<{ name: string }>) {
  const paths: Record<string, React.ReactNode> = {
    formula: <><path d="M5 5h6M5 19h6M8 5v14" /><path d="m15 8 5 8M20 8l-5 8" /></>,
    warning: <><path d="M12 3 2.8 19h18.4L12 3Z" /><path d="M12 9v4M12 16.5h.01" /></>,
    shield: <><path d="M12 3 4.5 6v5.6c0 4.5 3.1 7.7 7.5 9.4 4.4-1.7 7.5-4.9 7.5-9.4V6L12 3Z" /><path d="m8.5 12 2.2 2.2 4.8-5" /></>,
    document: <><path d="M6 2.8h8l4 4V21H6V2.8Z" /><path d="M14 2.8V7h4M9 12h6M9 16h4" /></>,
    search: <><circle cx="10.5" cy="10.5" r="6.5" /><path d="m15.5 15.5 5 5M10.5 7.5v3M10.5 13.5h.01" /></>,
    checklist: <><path d="M7 3h10v18H7z" /><path d="m9.5 8 1 1 2-2M14 8h1.5M9.5 13l1 1 2-2M14 13h1.5M9.5 18l1 1 2-2M14 18h1.5" /></>,
    upload: <><path d="M12 16V4M7.5 8.5 12 4l4.5 4.5" /><path d="M5 14v6h14v-6" /></>,
    settings: <><path d="M4 7h8M16 7h4M4 17h4M12 17h8M4 12h2M10 12h10" /><circle cx="14" cy="7" r="2" /><circle cx="8" cy="12" r="2" /><circle cx="10" cy="17" r="2" /></>,
    download: <><path d="M12 4v12M7.5 11.5 12 16l4.5-4.5" /><path d="M5 19h14" /></>,
    code: <><path d="m8.5 7-5 5 5 5M15.5 7l5 5-5 5M13.5 4l-3 16" /></>,
    receipt: <><path d="M6 3h12v18l-3-2-3 2-3-2-3 2V3Z" /><path d="M9 8h6M9 12h6M9 16h3" /></>,
  };

  return <svg {...sharedProps}>{paths[name] ?? paths.document}</svg>;
}
