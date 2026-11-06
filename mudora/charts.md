# Chart catalogue

Pick the chart from the question the numbers answer. Measurements go in a chart with values labelled on the marks. Don't add a table that repeats the chart. Source numbers in [brackets] are listed at the end.

## Ground rules for every chart

- **Position on a shared scale reads best.** It beats length, then angle, area and colour, in that order (Cleveland & McGill [1][2]). Prefer bars, dots and lines to pies, bubbles, treemaps and colour scales. NN/g: use bar, line or scatter in most cases [3].
- **Sort by value** unless the items have their own order, such as builds or device tiers [5][6].
- **Bars start at zero, always.** Dots and lines may zoom the axis, because they are read by position, not length [5][7][8].
- **Small multiples share their axes.** If they can't, say so in each panel title [9].
- **Same colour for the same thing on every chart of a page** [6]. Text stays in ink colours; the coloured mark beside it carries the identity. The reverse holds too: a colour never means two things. Before and after get their own pair, not the colours already used for Mac and Linux, so a reader never has to check the key to know which meaning a colour has on this card.
- **Every mark has a hover tip.** Bars, segments and stacked parts each show a short tip on hover and focus, with what the label leaves out: both values, the total, or why it is that size. Write the tip handler once, delegated on the document, so marks built by script get it too. A tip is held to the same plain words as the page: read it as a newcomer, and say what the number means for the run ("The run would take just as long without it"), not how the chart is drawn ("finishes before the jobs the run waits on"). A tip does not repeat what the reader already sees: no chart name, unit name, row label or card label, and no value already printed on the mark. "Macs: 122.9 of 131.9 min. Android, iOS and the unit tests." is right; "Run 1, main, Macs: 122.9 of 131.9 machine min" repeats the run, the row and the chart.
- **Every part of a stacked bar shows its value, and has its own tip.** Hide a part's value only when it measures too narrow on the rendered page, not by a fixed share; a missing number reads as missing data. Each part's tip says what that part is ("Macs: 14.7 of 89.6 min. Only the iOS build is left on a Mac"), not one tip for the whole bar.
- **No label leaves its card.** A value placed past the end of a short bar near the right edge goes on the bar's left side instead. Check the right end of every chart on the rendered page.
- **A timeline says in its title that its bars run side by side** ("jobs run side by side for 57 min"). Bars stacked in rows read as one after another unless the chart says otherwise.
- **Write the takeaway in the title or caption.** Don't make the reader work it out [6].
- **A chart reads without instructions.** If the caption or the text above has to explain how to read it (what the scale means, why the bars don't add up), the form is wrong. Change the chart rather than adding the explanation. A chart that only breaks up the text, without making a number easier to grasp, gets cut.
- **Values sit inside the bar.** Make bars tall enough to hold text (about 24px) and put the number in the middle of the fill; a bar too short to hold it gets the number just past its end. Never put values in a column at the far right, which the eye has to line up with each bar. On a before-and-after bar, the before value sits at the end of the before bar.
- **The unit goes on the values, once per page style.** When bar values carry their unit ("54.2 min"), every bar value on the page does, and titles, legends and axis ticks drop it ("Run time", not "Run time, minutes").
- **A key sits inside each card it explains.** When a legend covers several cards, give each card its own small key in its bottom-right corner, with its words faded until hover (solid on touch), so it stays beside the bars as the reader scrolls. Fade only the words, never the swatches: each swatch is the exact colour of its bar, from one shared colour definition, and check it on the rendered page. Don't float one key over the page or leave it above the first card.
- **A before-and-after bar explains itself.** The before bar stops a little short of the track end, so it reads as a bar and not the track. An after value of zero still shows a thin sliver of its colour. Both values carry the unit. Hovering the bar shows a tip that names both ("70 min today, 31 min with this change").
- **No dual axes, no radar charts, no broken axes, no 3D, no rainbow scales** (see "Avoid" below).

## Catalogue

Each entry: the question it answers · use it for · don't use it for · the rule that keeps it honest.

### Comparing amounts and ranking

| Chart | Question | Use it for | Don't use it for | Rule |
|---|---|---|---|---|
| **Bar** (horizontal for long labels) | How big is each, and which is biggest? | Median tap time per device, sorted | Values in a narrow band far from zero, such as pass rates of 94–99% [5]; spreads (a bar of averages hides them) [6] | Start at zero, never break the axis [5][7] |
| **Grouped bar** | How do values compare across two groupings? | p50 per device, iOS 17 and iOS 18 side by side | More than 3–4 series; use small multiples [6] | Bars in a group touch, with clear gaps between groups [6] |
| **Stacked / 100% stacked bar** | What is the total made of? | Pass / fail / skip per suite | Comparing middle segments, whose baselines move [10][11] | At most about 3 parts; put the important one on the baseline [11] |
| **Dot plot** | Compare or rank values that aren't near zero | Pass rate 91–99% per device, axis 90–100 | Counts where the total matters; bars show size better | The axis may start anywhere [5][8]; sort by value |
| **Lollipop** | Same as a bar, with less ink | Flake count across 30+ tests (avoids the buzz of many bars) [6] | A stick that doesn't start at zero | Zero baseline, sorted [10] |
| **Heatmap** | Patterns across two dimensions with too many cells for bars | Pass / fail for each test against each build | Reading exact values; colour is the weakest encoding [1][5] | One-hue scale for amounts, two hues only around a real midpoint, never a rainbow [6] |

### Range, spread and distribution

| Chart | Question | Use it for | Don't use it for | Rule |
|---|---|---|---|---|
| **Range / dumbbell** (min–max bar) | How far apart are two values per item? | Fastest to slowest of 10 runs per step, per device | 3+ points per row (use a strip plot) [12]; direction (use an arrow plot) [13] | Label both ends; zero is optional [13] |
| **Arrow plot** | Which way did each item move? | Before and after a fix, across many steps [13] | Only two or three items (a slope chart reads better) | Arrow points from before to after |
| **Strip / beeswarm** | What does every single run look like? | 10 runs per step, as points, so outliers show | Thousands of points per group (use a histogram) [15][6] | Jitter only sideways, never along the value axis [15] |
| **Histogram** | What shape is one distribution? | 500 cold starts; shows a two-hump cache hit / miss split | Overlaying several groups (use small multiples) [14] | Try a few bin widths before choosing [14][6] |
| **Box plot** | Median, middle half and outliers across many groups | Step time across 12 devices, technical readers | Small samples, two-hump data, or non-technical readers [15][6] | Show the sample size; overlay the points when n is small [15] |
| **Scatter** | How do two numbers relate? | Widget-tree size against response time | Implying cause [10]; dense data without transparency or binning [6] | Likely cause on x; size bubbles by area [6] |

### Change over time and before / after

| Chart | Question | Use it for | Don't use it for | Rule |
|---|---|---|---|---|
| **Line** | What's the trend over time or builds? | Memory over a 30-minute soak; nightly p90 over 60 builds | More than about 5 lines (highlight one, grey the rest, or split) [6]; unordered categories [16] | Zero optional [8][10]; don't flatten or exaggerate the slope [6] |
| **Slope chart** | How did each item change between two points? | Step time before vs after a fix, 8 steps | More than 2–3 time points [17][18] | Label values at both ends; colour improved vs got worse |
| **Area** | How does a total build up over time? | Heap over time, at most 3 stacked layers | Many stacked series [6][11]; exact values | Start at zero |
| **Small multiples** | Same chart repeated per group | One memory line per device in a grid | Panels with different scales and no warning [9] | Shared axes; logical order, such as worst first [9] |
| **Sparkline** | Recent trend next to a number | A stat tile "p90 tap 412 ms" with 30 builds behind it | Showing it without the number or any scale [19] | Mark the latest point; it matches the printed number [19] |

### Part of a whole, steps and flow

| Chart | Question | Use it for | Don't use it for | Rule |
|---|---|---|---|---|
| **Pie / donut** | What simple share of one whole? | "Two thirds of failures are timeouts", 2–3 slices | More than 5 slices, similar shares, several pies, overlapping categories [20][21] | Parts add to 100%; bars usually do it as well [21] |
| **Treemap** | How is a hierarchy split by size? | Bundle size by package, then module | Precise comparison [22][23] | About 2 levels; labels inside the boxes [22] |
| **Waterfall** | How do steps add up to a total? | A 1.2 s tap built from find, scroll, tap and settle | Steps that overlap in time [10] | First and total bars start at zero; colour ups and downs differently |
| **Timeline / span chart** | When did each phase start and end, and what overlapped? | Phases of one tap on a shared ms axis; requests during a screen load | Comparing durations across many runs (use dots or a box plot) | Time runs left to right from 0 [10] |
| **Sankey** | How does volume move between states? | 1,000 runs: queued → started → passed / failed / retried | Many stages, where bands cross [22] | Order nodes to minimise crossings [22] |

### Value against a target, and headline numbers

| Chart | Question | Use it for | Don't use it for | Rule |
|---|---|---|---|---|
| **Bullet chart** | Is this value good, OK or bad? | p90 cold start 1.8 s against a 2.0 s budget | More than 5 bands; values past the scale end [24][25] | Dark value bar, a target tick, 2–5 muted bands of one hue. Replaces gauges [24] |
| **Stat tile** | What's the one number, and is it OK? | "Pass rate 97.4%, up 1.2 points on last week" | A bare number with no comparison or time window [19] | Always a reference: change, unit, period. Add a sparkline if the trend matters |

## Avoid

- **Dual axes.** Two arbitrary scales can make any crossing or correlation appear, and readers were slower and less accurate with them [27]. Use two stacked panels on a shared time axis, or index both to a common start.
- **Radar / spider.** The axis order changes the shape and the filled area means nothing [6][26]. Use sorted bars or a dot plot per metric.
- **Broken axes.** Bar lengths can't be compared across the break and it looks deceptive [31][34].
- **Pie with more than 5 slices, gauges, 3D, rainbow colour scales.**

## Values that span a wide range (for example 12 ms to 5 s)

On one linear axis the 12 ms values shrink to slivers. In order of preference:

1. **Split into panels by size, each with its own linear axis**, such as "Inside the app, milliseconds" and "On the phone's screens, seconds". Bars stay honest and easy to read; say the scale in each panel title [27][31][33].
2. **Show where the time goes** with a waterfall or timeline, if the question is composition. Small steps should look small, because they are.
3. **Compare against a budget** (bullet chart per step, or % change from a baseline) when the question is regression or target [27].
4. **Log axis, with dots, not bars, for technical readers only.** Ticks in real units (10 ms, 100 ms, 1 s), the axis title says it's logarithmic, and a caption such as "each gridline is 10× the one before" [28][32]. Lay readers misread log charts: 56% read them correctly against 93% for linear in one study [30]. Bars on a log axis mean nothing [31].
5. **Table with small inline bars**, when the exact values matter more than the pattern [33].

Never use a broken axis for this.

## Quick decision guide

| The question | Use | Avoid |
|---|---|---|
| Compare or rank | Sorted bar; dot plot if far from zero; lollipop for many items | Pie, radar, 3D |
| Range or run-to-run spread | Range bar (min–max); strip plot (up to ~50 points); box plot for many groups and technical readers | Bar of averages with only error bars [6] |
| Shape of a distribution | Histogram | Box plot alone for two-hump data |
| Change over time | Line; small multiples past 5 series; sparkline inline | Dual axes, spaghetti |
| Before vs after | Slope chart (few items); arrow plot (many items) | Two pies |
| Part of a whole | 100% stacked bar (≤3 parts); pie for 2–3 parts; treemap for hierarchy | Pie with more than 5 slices |
| Steps of one operation | Waterfall (additive); timeline (start, end, overlap) | Pie of phases |
| Flow between states | Sankey (few stages) | Sankey with many crossings |
| Against a target | Bullet chart; stat tile with change | Gauges [24] |
| Relationship | Scatter, plus a trend line | Dual-axis line |
| Many cells over time | Heatmap | Rainbow palette |

## Sources

1. Cleveland & McGill ranking — https://info3312.infosci.cornell.edu/slides/04-graphical-perception.html ; https://www.cs.ubc.ca/~tmm/courses/533-07/slides/percep.pdf
2. FlowingData, Graphical perception — https://flowingdata.com/2010/03/20/graphical-perception-learn-the-fundamentals-first
3. NN/g, Choosing chart types — https://www.nngroup.com/videos/choosing-chart-types/
5. Wilke, Visualizing amounts — https://clauswilke.com/dataviz/visualizing-amounts.html
6. data-to-viz, Caveats — https://www.data-to-viz.com/caveats.html
7. Datawrapper, Why bar charts start at zero — https://www.datawrapper.de/academy/why-our-column-and-bar-charts-start-at-zero
8. ONS, Does the axis have to start at zero? — https://digitalblog.ons.gov.uk/2016/07/23/does-the-axis-have-to-start-at-zero-part-2-bar-charts
9. Wilke, Multi-panel figures — https://clauswilke.com/dataviz/multi-panel-figures.html
10. FT Visual Vocabulary — https://github.com/Financial-Times/chart-doctor/blob/main/visual-vocabulary/README.md
11. Wilke, Visualizing proportions — https://clauswilke.com/dataviz/visualizing-proportions.html
12. Domo, Dumbbell chart — https://domo-webflow.domo.com/learn/charts/dumbbell-plot-chart
13. Datawrapper, Arrow and range plots — https://datawrapper.de/blog/new-arrow-and-range-plots
14. Wilke, Histograms and density plots — https://clauswilke.com/dataviz/histograms-density-plots.html
15. Wilke, Boxplots and violins — https://clauswilke.com/dataviz/boxplots-violins.html
16. data-to-viz, Connect your dots — https://www.data-to-viz.com/caveat/connect_your_dot.html
17. Datawrapper, Slope charts — https://www.datawrapper.de/academy/how-to-create-a-slope-chart
18. UNHCR dataviz, Change over time — https://dataviz.unhcr.org/chart-types/change-over-time
19. Tufte, Sparkline theory and practice — https://www.edwardtufte.com/notebook/sparkline-theory-and-practice-edward-tufte/
20. Datawrapper, Pie charts — https://www.datawrapper.de/blog/pie-charts
21. Few, Save the Pies for Dessert — https://www.betterevaluation.org/sites/default/files/08-21-07.pdf
22. Wilke, Nested proportions — https://clauswilke.com/dataviz/nested-proportions.html
23. Fuselab (summarising NN/g on treemaps) — https://fuselabcreative.com/data-visualization-best-practices/
24. Few, Bullet graph design spec — https://www.perceptualedge.com/articles/misc/Bullet_Graph_Design_Spec.pdf
25. Domo, Bullet graphs — https://domo-webflow.domo.com/learn/charts/bullet-graphs
26. Significance, Off the radar — https://significancemagazine.com/off-the-radar-here-are-some-alternatives/
27. Datawrapper, Why not to use dual axis charts — https://www.datawrapper.de/blog/dualaxis/
28. Wilke, Axes and log scales — https://clauswilke.com/dataviz/coordinate-systems-axes.html
30. Harvard Taylor lab, log-scale misreading — https://taylorlab.oeb.harvard.edu/publications/logarithmic-scales-ecological-data-presentation-may-cause-misinterpretation
31. SAS, Broken Y-axis — https://blogs.sas.com/content/graphicallyspeaking/2012/05/31/broken-y-axis/
32. EU Data Visualisation Guide, Log scales — https://data.europa.eu/apps/data-visualisation-guide/logarithmic-scales
33. Cross Validated, alternatives to broken axes — https://stats.stackexchange.com/q/1764
34. Wikipedia, Misleading graph — https://en.wikipedia.org/wiki/Misleading_graph
