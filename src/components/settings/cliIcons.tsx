import type { SVGProps } from "react";

type IconProps = SVGProps<SVGSVGElement> & { size?: number };

// Claude icon — from simple-icons (CC0), official Anthropic Claude brand mark
// https://raw.githubusercontent.com/simple-icons/simple-icons/master/icons/claude.svg
export function ClaudeIcon(props: IconProps) {
  return (
    <svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" width={props.size ?? 20} height={props.size ?? 20} {...props}>
      <title>Claude Code</title>
      <path fill="#D97757" d="m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z"/>
    </svg>
  );
}

// Codex — OpenAI brand hexagon (used by Codex CLI, no separate CLI logo exists)
export function CodexIcon(props: IconProps) {
  return (
    <svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" width={props.size ?? 20} height={props.size ?? 20} {...props}>
      <title>Codex</title>
      <path fill="#10A37F" d="M11.99.867 1.195 6.932v12.136L11.99 25.07l10.795-6.002V6.932L11.99.867Zm-6.908 8.36a1.64 1.64 0 0 1 2.07-.644c.689.284 1.02.965.74 1.655-.269.666-.95 1.008-1.62.754a1.2 1.2 0 0 1-.764-.529l-.017-.025a1.19 1.19 0 0 1-.409-1.211Zm.81 4.918-.07-.026a1.19 1.19 0 0 1 .897.01c.663.258.973.952.69 1.61a1.201 1.201 0 0 1-1.587.701v-.001a1.201 1.201 0 0 1-.69-1.61 1.2 1.2 0 0 1 .164-.284l-.074.043a1.2 1.2 0 0 1 .67-.443Zm7.207 7.822a1.21 1.21 0 0 1-.913.182 1.2 1.2 0 0 1-.782-.536 1.198 1.198 0 0 1 1.512-1.674 1.2 1.2 0 0 1 .523.407c.187.246.264.57.209.875a1.2 1.2 0 0 1-.549.746Zm3.498-1.385a1.19 1.19 0 0 1-1.157-.334 1.22 1.22 0 0 1-.316-.558 1.204 1.204 0 0 1 1.534-1.435 1.2 1.2 0 0 1 .698.587 1.196 1.196 0 0 1-.76 1.74Zm.337-3.506a1.196 1.196 0 0 1-1.592.278 1.2 1.2 0 0 1-.51-.509 1.194 1.194 0 0 1 .177-1.357 1.2 1.2 0 0 1 .845-.425 1.2 1.2 0 0 1 .917.237 1.2 1.2 0 0 1 .44.847 1.2 1.2 0 0 1-.277.929Zm.434-3.965a1.198 1.198 0 0 1-1.543-1.008 1.22 1.22 0 0 1 .098-.64 1.202 1.202 0 0 1 1.946-.334 1.197 1.197 0 0 1 .154 1.246 1.2 1.2 0 0 1-.655.736Z"/>
    </svg>
  );
}

// OpenCode — official logo SVG from GitHub (MIT license)
// https://raw.githubusercontent.com/anomalyco/opencode/dev/packages/console/app/src/asset/logo-ornate-light.svg
export function OpenCodeIcon(props: IconProps) {
  return (
    <svg viewBox="0 0 234 42" fill="none" xmlns="http://www.w3.org/2000/svg" width={props.size ?? 20} height={(props.size ?? 20) * 42 / 234} {...props}>
      <title>OpenCode</title>
      <path d="M18 30H6V18H18V30Z" fill="#CFCECD"/>
      <path d="M18 12H6V30H18V12ZM24 36H0V6H24V36Z" fill="#656363"/>
      <path d="M48 30H36V18H48V30Z" fill="#CFCECD"/>
      <path d="M36 30H48V12H36V30ZM54 36H36V42H30V6H54V36Z" fill="#656363"/>
      <path d="M84 24V30H66V24H84Z" fill="#CFCECD"/>
      <path d="M84 24H66V30H84V36H60V6H84V24ZM66 18H78V12H66V18Z" fill="#656363"/>
      <path d="M108 36H96V18H108V36Z" fill="#CFCECD"/>
      <path d="M108 12H96V36H90V6H108V12ZM114 36H108V12H114V36Z" fill="#656363"/>
      <path d="M144 30H126V18H144V30Z" fill="#CFCECD"/>
      <path d="M144 12H126V30H144V36H120V6H144V12Z" fill="#211E1E"/>
      <path d="M168 30H156V18H168V30Z" fill="#CFCECD"/>
      <path d="M168 12H156V30H168V12ZM174 36H150V6H174V36Z" fill="#211E1E"/>
      <path d="M198 30H186V18H198V30Z" fill="#CFCECD"/>
      <path d="M198 12H186V30H198V12ZM204 36H180V6H198V0H204V36Z" fill="#211E1E"/>
      <path d="M234 24V30H216V24H234Z" fill="#CFCECD"/>
      <path d="M216 12V18H228V12H216ZM234 24H216V30H234V36H210V6H234V24Z" fill="#211E1E"/>
    </svg>
  );
}

// Google Gemini — from simple-icons (CC0), official Google Gemini icon
// https://raw.githubusercontent.com/simple-icons/simple-icons/master/icons/googlegemini.svg
export function GeminiIcon(props: IconProps) {
  return (
    <svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" width={props.size ?? 20} height={props.size ?? 20} {...props}>
      <title>Google Gemini</title>
      <path fill="url(#geminiclione-google)" d="M11.04 19.32Q12 21.51 12 24q0-2.49.93-4.68.96-2.19 2.58-3.81t3.81-2.55Q21.51 12 24 12q-2.49 0-4.68-.93a12.3 12.3 0 0 1-3.81-2.58 12.3 12.3 0 0 1-2.58-3.81Q12 2.49 12 0q0 2.49-.96 4.68-.93 2.19-2.55 3.81a12.3 12.3 0 0 1-3.81 2.58Q2.49 12 0 12q2.49 0 4.68.96 2.19.93 3.81 2.55t2.55 3.81"/>
      <defs>
        <linearGradient id="geminiclione-google" x1="0" y1="0" x2="24" y2="24" gradientUnits="userSpaceOnUse">
          <stop stopColor="#4285F4"/>
          <stop offset="1" stopColor="#9B72CB"/>
        </linearGradient>
      </defs>
    </svg>
  );
}

// Kiro — based on official kiro-icon.png from their GitHub (no SVG available)
// https://github.com/kirodotdev/Kiro/blob/main/assets/kiro-icon.png
export function KiroIcon(props: IconProps) {
  return (
    <svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" width={props.size ?? 20} height={props.size ?? 20} {...props}>
      <title>Kiro</title>
      <path fill="#FF9900" d="M13.5.5 5.2 13.8h4.5L7 23.5 15.5 10h-4.5L13.5.5Z"/>
      <path fill="#FFB84D" d="M12.8 1 5.1 13h4l-2.5 9.5L15 10.5h-4L12.8 1Z"/>
    </svg>
  );
}

// CodeBuddy (Tencent) — based on VS Code extension icon, no official public SVG
// https://marketplace.visualstudio.com/items?itemName=Tencent-Cloud.coding-copilot
export function CodeBuddyIcon(props: IconProps) {
  return (
    <svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" width={props.size ?? 20} height={props.size ?? 20} {...props}>
      <title>CodeBuddy</title>
      <defs>
        <linearGradient id="cb-grad" x1="0" y1="0" x2="24" y2="24" gradientUnits="userSpaceOnUse">
          <stop stopColor="#0052D9"/>
          <stop offset="1" stopColor="#0066FF"/>
        </linearGradient>
      </defs>
      <rect x="2" y="3" width="20" height="15" rx="3" fill="url(#cb-grad)"/>
      <path d="M7 18l5 3 5-3H7Z" fill="url(#cb-grad)"/>
      <path d="M7.5 8h3.5M7.5 11h6M7.5 14h4" stroke="white" strokeWidth="1.2" strokeLinecap="round"/>
    </svg>
  );
}

// Qoder — no official SVG/PNG logo found on docs.qoder.com or qoder.com
export function QoderIcon(props: IconProps) {
  return (
    <svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" width={props.size ?? 20} height={props.size ?? 20} {...props}>
      <title>Qoder</title>
      <circle cx="11" cy="11" r="9" fill="#7C3AED"/>
      <path d="M8.5 8.5h3.5c1.2 0 2 .8 2 2s-.8 2-2 2H8.5V8.5ZM8.5 12.5h4v2h-4v-2Z" fill="white"/>
      <path d="M15 15l4.5 4.5" stroke="#7C3AED" strokeWidth="2.5" strokeLinecap="round"/>
    </svg>
  );
}

export const CLI_ICON_MAP: Record<string, React.FC<IconProps>> = {
  claude: ClaudeIcon,
  codex: CodexIcon,
  opencode: OpenCodeIcon,
  gemini: GeminiIcon,
  kiro: KiroIcon,
  codebuddy: CodeBuddyIcon,
  qoder: QoderIcon,
};
