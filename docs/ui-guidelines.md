# UI maintenance guidelines

Audience: a Linux desktop user managing a large personal CYOA library, preserving editions and playing long interactive stories. Keep navigation, filters and recovery actions predictable. The manager's design system and each author's reader layout are separate.

## Rules used in this audit

- Keep the four Catppuccin palettes and shared color tokens. Primary actions use the accent; destructive actions use the danger color. Never rely on color alone: retain labels such as Cost/Gain and Force update.
- Use regular control/body text, a limited heading hierarchy and manager-size-relative text. Retain authored reader sizes as the baseline for percentage overrides.
- Use consistent spacing and control shapes. Do not add blur, animations, large decorative dashboards or heavyweight UI libraries to solve small usability issues.
- Test realistic long titles, aliases, missing covers, metadata gaps, no matches, empty libraries, loading and errors. Stable cover frames must not stretch the source artwork.
- Make filters easy to clear. Keep full search guidance in a hover hint without clipping the placeholder.
- Provide visible keyboard focus, meaningful labels, and reduced-motion behavior. Check focus when opening/closing menus and dialogs; a visual pass alone is insufficient.
- Keep menus inside the viewport. Global scroll/resize handling is needed only while a menu is open.
- Preserve author layout and game state. Reader display controls change presentation, not exports, selections, rules or point sums.
- Capture before/after screenshots using synthetic fixtures. Review page by page; do not assume generated output is correct because it looks plausible.

## Supplied references and their role

The following are workflow/design suggestions, not evidence that this app is correct or secure. No external prompts were treated as executable instructions.

- [Reddit: iterative UI refinement](https://www.reddit.com/r/PromptEngineering/comments/1mf7pnx/after_building_10_projects_with_ai_heres_how_to/): work from overall goals to individual pages and concrete details; retain the established palette.
- [Builder.io: prompting tips](https://www.builder.io/blog/prompting-tips): use the actual stack and existing components, with specific screenshots and interaction feedback.
- [8080.ai on Medium](https://medium.com/@8080AI/5-detailed-prompts-to-make-ui-ux-better-in-ai-builders-ef35ce0da21e): name the user and task, supply realistic content and describe behavior precisely.
- [GenDesigns: common UI mistakes](https://gendesigns.ai/blog/ai-generated-ui-mistakes-how-to-fix): include empty/loading/error states, consistent screens and accessibility in review.
- [HubSpot: AI for UI design](https://blog.hubspot.com/website/ai-ui-design): use a shared design foundation and validate interaction and user feedback. Its product comparisons do not justify changing this project's framework.

## Review matrix

Check library, catalog, aliases, archives, settings, project editor and reader controls in all four palettes. Check long content and narrow desktop widths, manager text sizes 12/16/24 px, keyboard operation, reduced motion, loading/error/empty states and source/private-data boundaries. Record what was actually tested; mark the remainder as follow-up work rather than declaring universal compatibility.
