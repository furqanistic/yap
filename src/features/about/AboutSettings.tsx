import { useState } from "react";
import { IconBugMinimalistic } from "@devigner-ui/icons/BugMinimalistic";
import { IconCheck } from "@devigner-ui/icons/Check";
import { IconCodeSquare } from "@devigner-ui/icons/CodeSquare";
import { IconCopy } from "@devigner-ui/icons/Copy";
import { IconSquareShareLine } from "@devigner-ui/icons/SquareShareLine";
import { Page } from "@/components/page";
import { SettingRow, SettingsGroup } from "@/components/settings";
import { Button } from "@/components/ui";
import { openExternalUrl } from "@/lib/opener";
import { getPlatform } from "@/lib/platform";
import { APP_INFO } from "./options";
import "./AboutSettings.css";

export function AboutSettings() {
  const [copied, setCopied] = useState(false);

  const handleCopySystemInfo = async () => {
    const info = `${APP_INFO.name} v${APP_INFO.version}\nOS: ${getPlatform()}\nUser Agent: ${navigator.userAgent}`;
    try {
      await navigator.clipboard.writeText(info);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error("Failed to copy system info:", error);
    }
  };

  return (
    <Page title="About" description="Version, license, and open-source credits.">
      <SettingsGroup title="Application">
        <SettingRow
          title="Version"
          description="Current installed release."
          hint="Early development preview"
        >
          {(a11y) => (
            <div className="about-actions">
              <span className="about-version-badge">v{APP_INFO.version}</span>
              <Button
                icon={copied ? IconCheck : IconCopy}
                onClick={handleCopySystemInfo}
                title="Copy version and system details"
                {...a11y}
              >
                {copied ? "Copied" : "Copy details"}
              </Button>
            </div>
          )}
        </SettingRow>

        <SettingRow
          title="Updates"
          description="Look for new releases on GitHub."
        >
          {(a11y) => (
            <Button
              icon={IconSquareShareLine}
              onClick={() => openExternalUrl(APP_INFO.releasesUrl)}
              {...a11y}
            >
              Check releases
            </Button>
          )}
        </SettingRow>
      </SettingsGroup>

      <SettingsGroup title="Open source">
        <SettingRow
          title="Source code"
          description="View the source repository and join development."
        >
          {(a11y) => (
            <Button
              icon={IconCodeSquare}
              onClick={() => openExternalUrl(APP_INFO.repositoryUrl)}
              {...a11y}
            >
              GitHub
            </Button>
          )}
        </SettingRow>

        <SettingRow
          title="License"
          description="Distributed under the open-source MIT License."
        >
          {(a11y) => (
            <Button
              icon={IconSquareShareLine}
              onClick={() => openExternalUrl(APP_INFO.licenseUrl)}
              {...a11y}
            >
              View license
            </Button>
          )}
        </SettingRow>

        <SettingRow
          title="Third-party notices"
          description="Attributions and licenses for open-source dependencies."
        >
          {(a11y) => (
            <Button
              icon={IconSquareShareLine}
              onClick={() => openExternalUrl(APP_INFO.thirdPartyUrl)}
              {...a11y}
            >
              View notices
            </Button>
          )}
        </SettingRow>
      </SettingsGroup>

      <SettingsGroup title="Community & support">
        <SettingRow
          title="Report an issue"
          description="Open a bug report or suggest an enhancement on GitHub."
        >
          {(a11y) => (
            <Button
              icon={IconBugMinimalistic}
              onClick={() => openExternalUrl(APP_INFO.issuesUrl)}
              {...a11y}
            >
              Open issue
            </Button>
          )}
        </SettingRow>

        <SettingRow
          title="Documentation"
          description="Read setup guides and usage documentation."
        >
          {(a11y) => (
            <Button
              icon={IconSquareShareLine}
              onClick={() => openExternalUrl(APP_INFO.docsUrl)}
              {...a11y}
            >
              Read docs
            </Button>
          )}
        </SettingRow>
      </SettingsGroup>
    </Page>
  );
}
