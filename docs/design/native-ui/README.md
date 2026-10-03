# Native UI layout proposals

These GPT Image sketches are design proposals, not screenshots of the running application. All layouts include hardware status, power profiles, automatic/manual/maximum fan controls, startup preference/status, close-to-tray preference, device information, tray access and exit.

A was selected. Subsequent decisions remove implementation copy, the refresh button and bottom tray/exit buttons; place the current profile beside its options; add green/blue/red mode colors, system light/dark switching, and a close-choice dialog with Remember. The running implementation is described in [the UI guide](../../native-ui.md).

- [A — two-column dashboard](a-dashboard.png): balanced status and controls, with secondary settings on the right.
- [B — compact vertical panel](b-compact.png): one column and minimal navigation.
- [C — wide workspace](c-workspace.png): navigation rail and three working areas.

Fan slider commits on release, without an Apply button. Keyboard changes should be coalesced. Preserve range validation and all cooling safeguards. Values in the sketches are illustrative, not live readings. Native control appearance will follow the installed Windows theme; exact rendering may differ from the generated sketches.

The original prompts are saved in [prompts.md](prompts.md). C was edited once to add the initially omitted Maximum option while preserving its layout.
