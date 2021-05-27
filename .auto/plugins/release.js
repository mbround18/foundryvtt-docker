const {
    determineNextVersion,
    execPromise,
    getCurrentBranch,
    DEFAULT_PRERELEASE_BRANCHES,
} = require( "@auto-it/core");
const { inc } = require("semver");

module.exports = class GitTagPlugin {
    /** The name of the plugin */
    name = "release";

    /** Tap into auto plugin points. */
    apply(auto) {  /** Get the latest tag in the repo, if none then the first commit */
    async function getTag() {
        try {
            return await auto && auto.git && auto.git.getLatestTagInBranch();
        } catch (error) {
            return auto.prefixRelease("0.0.0");
        }
    }

        auto.hooks.getPreviousVersion.tapPromise(this.name, async () => {
            if (!auto.git) {
                throw new Error(
                    "Can't calculate previous version without Git initialized!"
                );
            }

            return getTag();
        });

        auto.hooks.version.tapPromise(
            this.name,
            async ({ bump, dryRun, quiet }) => {
                if (!auto.git) {
                    return;
                }

                const lastTag = await getTag();
                const newTag = inc(lastTag, bump);

                if (!newTag) {
                    auto.logger.log.info("No release found, doing nothing");
                    return;
                }

                const prefixedTag = auto.prefixRelease(newTag);

                if (dryRun) {
                    if (quiet) {
                        console.log(prefixedTag);
                    } else {
                        auto.logger.log.info(`Would have published: ${prefixedTag}`);
                    }

                    return;
                }

                auto.logger.log.info(`Tagging new tag: ${lastTag} => ${prefixedTag}`);
                await execPromise("git", [
                    "tag",
                    prefixedTag,
                    "-m",
                    `"Update version to ${prefixedTag}"`,
                ]);
            }
        );

        auto.hooks.next.tapPromise(
            this.name,
            async (preReleaseVersions, { bump, dryRun }) => {
                if (!auto.git) {
                    return preReleaseVersions;
                }

                const prereleaseBranches =
                    auto.config?.prereleaseBranches ?? DEFAULT_PRERELEASE_BRANCHES;
                const branch = getCurrentBranch() || "";
                const prereleaseBranch = prereleaseBranches.includes(branch)
                    ? branch
                    : prereleaseBranches[0];
                const lastRelease = await auto.git.getLatestRelease();
                const current =
                    (await auto.git.getLastTagNotInBaseBranch(prereleaseBranch)) ||
                    (await auto.getCurrentVersion(lastRelease));
                const prerelease = auto.prefixRelease(
                    determineNextVersion(lastRelease, current, bump, prereleaseBranch)
                );

                preReleaseVersions.push(prerelease);

                if (dryRun) {
                    return preReleaseVersions;
                }

                await execPromise("git", [
                    "tag",
                    prerelease,
                    "-m",
                    `"Tag pre-release: ${prerelease}"`,
                ]);
                await execPromise("git", ["push", auto.remote, branch, "--tags"]);

                return preReleaseVersions;
            }
        );

        auto.hooks.publish.tapPromise(this.name, async () => {
            auto.logger.log.info("Pushing new tag to GitHub");
            if (process.env.GITHUB_REPOSITORY.length === 0) {
                throw new Error("Failed to find GITHUB_REPOSITORY in environment!")
            }

            await execPromise("git", [
                "remote",
                "set-url",
                "origin",
                `git@github.com:${process.env.GITHUB_REPOSITORY}.git`,
            ]);

            await execPromise("git", [
                "push",
                "--follow-tags",
                "--set-upstream",
                "origin",
                getCurrentBranch() || auto.baseBranch,
            ]);
        });
    }
}
