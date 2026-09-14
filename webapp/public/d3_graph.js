// ==========================================================================
// D3.js Collapsible Tree & Visual Hierarchy Visualizer
// Replaces force-directed physics with clean, intuitive hierarchical cards
// ==========================================================================

let treeSvg = null;
let treeG = null;
let treeZoom = null;
let currentRootData = null;
let treeRoot = null;
let activeWordEntry = null;

// Branch color palette - Support both Light and Dark themes dynamically
const TREE_COLORS_LIGHT = {
  pie_root: { bg: "#FEF3C7", border: "#D97706", badgeBg: "#D97706", badgeText: "#FFFFFF", text: "#92400E", line: "#D97706" },
  eastern: { bg: "#ECFDF5", border: "#059669", badgeBg: "#059669", badgeText: "#FFFFFF", text: "#065F46", line: "#059669" },
  indic: { bg: "#F0F9FF", border: "#0284C7", badgeBg: "#0284C7", badgeText: "#FFFFFF", text: "#0369A1", line: "#0284C7" },
  thai: { bg: "#ECFDF5", border: "#047857", badgeBg: "#047857", badgeText: "#FFFFFF", text: "#064E3B", line: "#047857" },
  western: { bg: "#EEF2FF", border: "#4F46E5", badgeBg: "#4F46E5", badgeText: "#FFFFFF", text: "#3730A3", line: "#4F46E5" },
  germanic: { bg: "#F0F9FF", border: "#0284C7", badgeBg: "#0284C7", badgeText: "#FFFFFF", text: "#075985", line: "#0284C7" },
  italic: { bg: "#FAF5FF", border: "#9333EA", badgeBg: "#9333EA", badgeText: "#FFFFFF", text: "#6B21A8", line: "#9333EA" },
  greek: { bg: "#FFF1F2", border: "#E11D48", badgeBg: "#E11D48", badgeText: "#FFFFFF", text: "#9F1239", line: "#E11D48" },
  english: { bg: "#F8FAFC", border: "#236596", badgeBg: "#236596", badgeText: "#FFFFFF", text: "#0F2942", line: "#236596" },
  default: { bg: "#F8FAFC", border: "#64748B", badgeBg: "#64748B", badgeText: "#FFFFFF", text: "#1E293B", line: "#64748B" }
};

const TREE_COLORS_DARK = {
  pie_root: { bg: "#2A1F10", border: "#F59E0B", badgeBg: "#D97706", badgeText: "#FFFFFF", text: "#FDE68A", line: "#F59E0B" },
  eastern: { bg: "#08231A", border: "#10B981", badgeBg: "#059669", badgeText: "#FFFFFF", text: "#A7F3D0", line: "#10B981" },
  indic: { bg: "#092237", border: "#38BDF8", badgeBg: "#0284C7", badgeText: "#FFFFFF", text: "#BAE6FD", line: "#38BDF8" },
  thai: { bg: "#09261E", border: "#34D399", badgeBg: "#047857", badgeText: "#FFFFFF", text: "#D1FAE5", line: "#34D399" },
  western: { bg: "#191B36", border: "#818CF8", badgeBg: "#4F46E5", badgeText: "#FFFFFF", text: "#C7D2FE", line: "#818CF8" },
  germanic: { bg: "#092237", border: "#38BDF8", badgeBg: "#0284C7", badgeText: "#FFFFFF", text: "#BAE6FD", line: "#38BDF8" },
  italic: { bg: "#231333", border: "#C084FC", badgeBg: "#9333EA", badgeText: "#FFFFFF", text: "#E9D5FF", line: "#C084FC" },
  greek: { bg: "#2E121C", border: "#FB7185", badgeBg: "#E11D48", badgeText: "#FFFFFF", text: "#FECDD3", line: "#FB7185" },
  english: { bg: "#0F1E30", border: "#38BDF8", badgeBg: "#236596", badgeText: "#FFFFFF", text: "#F8FAFC", line: "#38BDF8" },
  default: { bg: "#1E293B", border: "#94A3B8", badgeBg: "#64748B", badgeText: "#FFFFFF", text: "#F8FAFC", line: "#94A3B8" }
};

function getTreeColors(styleKey) {
  const isDark = document.body.classList.contains('theme-dark');
  const palette = isDark ? TREE_COLORS_DARK : TREE_COLORS_LIGHT;
  return palette[styleKey] || palette.default;
}

// Transparent Proxy so all TREE_COLORS[key] lookups automatically resolve based on current theme
const TREE_COLORS = new Proxy({}, {
  get(target, prop) {
    return getTreeColors(prop);
  }
});

function initD3Graph() {
  const svgEl = document.getElementById("d3GraphSvg");
  if (!svgEl) return;

  treeSvg = d3.select("#d3GraphSvg");
  treeSvg.selectAll("*").remove();

  // Define SVG Gradients and Filters
  const defs = treeSvg.append("defs");

  // Subtle Drop Shadow Filter for Light Mode
  const filter = defs.append("filter")
    .attr("id", "cardShadow")
    .attr("x", "-20%")
    .attr("y", "-20%")
    .attr("width", "140%")
    .attr("height", "140%");
  filter.append("feDropShadow")
    .attr("dx", 0)
    .attr("dy", 3)
    .attr("stdDeviation", 5)
    .attr("flood-color", "#0F2942")
    .attr("flood-opacity", 0.08);

  // Setup Zoom & Pan Container
  treeG = treeSvg.append("g").attr("class", "tree-main-group");

  treeZoom = d3.zoom()
    .scaleExtent([0.35, 2.2])
    .on("zoom", (event) => {
      treeG.attr("transform", event.transform);
    });

  treeSvg.call(treeZoom);

  // Toolbar Button Handlers
  const resetBtn = document.getElementById("graphResetZoomBtn");
  if (resetBtn) {
    resetBtn.onclick = () => resetTreeView();
  }

  const expandAllBtn = document.getElementById("graphExpandAllBtn");
  if (expandAllBtn) {
    expandAllBtn.onclick = () => expandAllNodes();
  }

  const collapseAllBtn = document.getElementById("graphCollapseAllBtn");
  if (collapseAllBtn) {
    collapseAllBtn.onclick = () => collapseAllNodes();
  }

  const toggleViewBtn = document.getElementById("graphToggleViewBtn");
  if (toggleViewBtn) {
    toggleViewBtn.onclick = () => toggleGraphicOrCardView();
  }
}

/**
 * Builds a friendly hierarchical tree structure from word entry and graph data
 */
function buildTreeHierarchy(entry, graphData) {
  if (!entry) return null;

  const pieWord = entry.pie_root || (graphData && graphData.pie_root) || "*PIE";
  const pieMeaning = entry.pie_meaning || "ความหมายดั้งเดิม";
  const indicWord = entry.pali_sanskrit_form || "ภาษาสันสกฤต/บาลี";
  const indicMeaning = entry.pali_sanskrit_meaning || "";
  const thaiWord = entry.thai_word;
  const thaiRead = entry.orst_read ? `[${entry.orst_read}]` : "";
  const thaiPos = entry.orst_pos || "น.";
  const thaiDef = entry.orst_definition || "";

  // Categorize English cognates into subfamilies
  const germanicCogs = [];
  const latinCogs = [];
  const greekCogs = [];
  const otherCogs = [];

  (entry.english_cognates || []).forEach((cog, i) => {
    const orig = (cog.origin_language || "").toLowerCase();
    const item = {
      id: `cog_${i}_${cog.word}`,
      name: cog.word,
      type: "english_leaf",
      icon: "EN",
      badge: cog.difficulty || "General",
      detail: cog.origin_language || "English",
      subDetail: cog.derivation_path || "",
      note: cog.usage_note || "",
      era: "คริสต์ศตวรรษที่ 5 - ปัจจุบัน",
      styleKey: "english",
      category: "คำในภาษาอังกฤษ"
    };

    if (orig.includes("greek") || orig.includes("hellenic")) {
      item.icon = "GRC";
      greekCogs.push(item);
    } else if (orig.includes("latin") || orig.includes("french") || orig.includes("italic") || orig.includes("romance")) {
      item.icon = "LAT";
      latinCogs.push(item);
    } else if (orig.includes("german") || orig.includes("old english") || orig.includes("norse")) {
      item.icon = "GER";
      germanicCogs.push(item);
    } else {
      otherCogs.push(item);
    }
  });

  const westernSubBranches = [];
  if (germanicCogs.length > 0) {
    westernSubBranches.push({
      id: "branch_germanic",
      name: "สายเจอร์แมนิก (Germanic)",
      type: "branch",
      icon: "GER",
      badge: `${germanicCogs.length} คำ`,
      detail: "สายตระกูลภาษาอังกฤษพื้นถิ่น",
      era: "~500 BCE",
      styleKey: "germanic",
      children: germanicCogs
    });
  }

  if (latinCogs.length > 0) {
    westernSubBranches.push({
      id: "branch_italic",
      name: "สายอิตาลิก / ละติน (Italic/Latin)",
      type: "branch",
      icon: "LAT",
      badge: `${latinCogs.length} คำ`,
      detail: "สายคำศัพท์วิชาการและการศึกษา",
      era: "~750 BCE",
      styleKey: "italic",
      children: latinCogs
    });
  }

  if (greekCogs.length > 0) {
    westernSubBranches.push({
      id: "branch_greek",
      name: "สายเฮลเลนิก / กรีก (Hellenic/Greek)",
      type: "branch",
      icon: "GRC",
      badge: `${greekCogs.length} คำ`,
      detail: "สายปรัชญาและศัพท์เฉพาะทาง",
      era: "~800 BCE",
      styleKey: "greek",
      children: greekCogs
    });
  }

  if (otherCogs.length > 0) {
    westernSubBranches.push({
      id: "branch_other",
      name: "สายร่วมอื่น ๆ (Other Branches)",
      type: "branch",
      icon: "EUR",
      badge: `${otherCogs.length} คำ`,
      detail: "คำร่วมตระกูลอินโด-ยูโรเปียน",
      era: "ประวัติศาสตร์สากล",
      styleKey: "western",
      children: otherCogs
    });
  }

  // Eastern Branch (Indo-Aryan -> Thai)
  const easternBranch = {
    id: "branch_eastern",
    name: "สายตะวันออก (อินโด-อารยัน)",
    type: "branch",
    icon: "SAN",
    badge: "สู่ภาษาไทย",
    detail: "สายสัทศาสตร์ Satem ทางเอเชียใต้",
    era: "~2000 BCE",
    styleKey: "eastern",
    children: [
      {
        id: "node_indic",
        name: indicWord,
        type: "intermediate_lang",
        icon: "IND",
        badge: entry.pali_sanskrit_lang || "สันสกฤต/บาลี",
        detail: `ความหมายเดิม: "${indicMeaning}"`,
        subDetail: "ภาษาคัมภีร์พระเวทและพุทธศาสนา",
        era: "~1500 BCE",
        styleKey: "indic",
        children: [
          {
            id: "node_thai",
            name: thaiWord,
            type: "thai_leaf",
            icon: "TH",
            badge: `${thaiPos} คำยืม`,
            detail: `${thaiRead} ${thaiPos}`,
            subDetail: thaiDef ? (thaiDef.length > 45 ? thaiDef.slice(0, 45) + "..." : thaiDef) : "พจนานุกรมราชบัณฑิตยสภา",
            era: "สุโขทัย - ปัจจุบัน",
            styleKey: "thai"
          }
        ]
      }
    ]
  };

  // Western Branch (Centum -> European / English)
  const westernBranch = {
    id: "branch_western",
    name: "สายตะวันตก (ยุโรป/อังกฤษ)",
    type: "branch",
    icon: "EUR",
    badge: `${(entry.english_cognates || []).length} คำร่วมสาย`,
    detail: "สายสัทศาสตร์ Centum ทางยุโรป",
    era: "~1000 BCE",
    styleKey: "western",
    children: westernSubBranches
  };

  // Root PIE Node
  return {
    id: "pie_root",
    name: pieWord,
    type: "pie_root",
    icon: "PIE",
    badge: "รากบรรพบุรุษร่วม (PIE)",
    detail: `ความหมาย: "${pieMeaning}"`,
    subDetail: "ทุ่งหญ้าสเตปป์ยูเรเซียโบราณ",
    era: "~4500-2500 BCE (~6,000 ปีก่อน)",
    styleKey: "pie_root",
    children: [easternBranch, westernBranch]
  };
}

/**
 * Main rendering function called by app.js
 */
function renderD3Graph(graphData) {
  if (!treeSvg) initD3Graph();

  activeWordEntry = window.activeEntry || null;
  const rawTree = buildTreeHierarchy(activeWordEntry, graphData);
  if (!rawTree) return;

  currentRootData = rawTree;

  // Set titles and badges
  const titleEl = document.getElementById("graphTitle");
  if (titleEl) {
    titleEl.innerText = `ผังต้นไม้รากศัพท์ของคำว่า "${rawTree.children[0]?.children[0]?.children[0]?.name || graphData.thai_word}" (ราก ${rawTree.name})`;
  }

  // Update Breadcrumb to Root
  updateBreadcrumb([rawTree.name]);

  // Convert to D3 hierarchy
  treeRoot = d3.hierarchy(currentRootData, d => d.children);
  treeRoot.x0 = 280;
  treeRoot.y0 = 40;
  window.treeRoot = treeRoot;

  // Render Graphic Tree
  updateTree(treeRoot);

  // Render Cascading Cards View for mobile/toggle
  renderCascadingCardView(currentRootData);

  // Focus root node initially
  selectNodeDetails(rawTree);
  resetTreeView();
}

/**
 * Updates the collapsible tree with smooth card rendering & bezier curves
 */
function updateTree(source) {
  const cardWidth = 230;
  const cardHeight = 68;
  const dx = 88; // vertical separation between sibling cards
  const dy = 270; // horizontal separation between levels

  // Compute the new tree layout
  const treeLayout = d3.tree().nodeSize([dx, dy]);
  treeLayout(treeRoot);

  const nodes = treeRoot.descendants();
  const links = treeRoot.links();

  // Links Selection
  const linkGroup = treeG.selectAll(".tree-links-group").data([0]);
  const linkGroupEnter = linkGroup.enter().append("g").attr("class", "tree-links-group");
  const linkContainer = linkGroup.merge(linkGroupEnter);

  const link = linkContainer.selectAll(".tree-link")
    .data(links, d => d.target.data.id);

  // Enter any new links at the parent's previous position
  const linkEnter = link.enter().append("path")
    .attr("class", "tree-link")
    .attr("d", d => {
      const o = { x: source.x0 || 280, y: source.y0 || 40 };
      return diagonalCurve({ source: o, target: o }, cardWidth);
    })
    .attr("fill", "none")
    .attr("stroke", d => {
      const style = TREE_COLORS[d.target.data.styleKey] || TREE_COLORS.default;
      return style.border;
    })
    .attr("stroke-width", 2)
    .attr("stroke-opacity", 0.45)
    .attr("stroke-dasharray", d => d.target.data.type === "branch" ? "none" : "none");

  // Transition links to their new position
  link.merge(linkEnter).transition().duration(400)
    .attr("d", d => diagonalCurve(d, cardWidth))
    .attr("stroke", d => {
      const style = TREE_COLORS[d.target.data.styleKey] || TREE_COLORS.default;
      return style.border;
    })
    .attr("stroke-opacity", 0.55);

  // Transition exiting nodes to the parent's new position
  link.exit().transition().duration(400)
    .attr("d", d => {
      const o = { x: source.x, y: source.y };
      return diagonalCurve({ source: o, target: o }, cardWidth);
    })
    .attr("stroke-opacity", 0)
    .remove();

  // Nodes Selection
  const nodeGroup = treeG.selectAll(".tree-nodes-group").data([0]);
  const nodeGroupEnter = nodeGroup.enter().append("g").attr("class", "tree-nodes-group");
  const nodeContainer = nodeGroup.merge(nodeGroupEnter);

  const node = nodeContainer.selectAll(".tree-node-card")
    .data(nodes, d => d.data.id);

  // Enter new nodes at the parent's previous position
  const nodeEnter = node.enter().append("g")
    .attr("class", "tree-node-card")
    .attr("transform", d => `translate(${source.y0 || 40},${source.x0 || 280})`)
    .style("cursor", "pointer")
    .on("click", (event, d) => handleNodeClick(event, d));

  // Outer Card Box (Card Background & Glow)
  nodeEnter.append("rect")
    .attr("class", "card-bg")
    .attr("width", cardWidth)
    .attr("height", cardHeight)
    .attr("y", -cardHeight / 2)
    .attr("rx", 10)
    .attr("filter", "url(#cardShadow)")
    .attr("fill", d => (TREE_COLORS[d.data.styleKey] || TREE_COLORS.default).bg)
    .attr("stroke", d => (TREE_COLORS[d.data.styleKey] || TREE_COLORS.default).border)
    .attr("stroke-width", 1.5);

  // Card Left Accent Stripe
  nodeEnter.append("rect")
    .attr("class", "card-accent-bar")
    .attr("width", 5)
    .attr("height", cardHeight - 14)
    .attr("x", 4)
    .attr("y", -(cardHeight - 14) / 2)
    .attr("rx", 3)
    .attr("fill", d => (TREE_COLORS[d.data.styleKey] || TREE_COLORS.default).border);

  // Icon Badge Circle
  nodeEnter.append("circle")
    .attr("cx", 26)
    .attr("cy", 0)
    .attr("r", 13)
    .attr("fill", d => (TREE_COLORS[d.data.styleKey] || TREE_COLORS.default).badgeBg)
    .attr("stroke", d => (TREE_COLORS[d.data.styleKey] || TREE_COLORS.default).border)
    .attr("stroke-width", 1.2);

  // Icon Text (Language Code Tag)
  nodeEnter.append("text")
    .attr("class", "card-icon")
    .attr("x", 26)
    .attr("y", 4)
    .attr("text-anchor", "middle")
    .attr("font-family", "'Sarabun', 'Krub', sans-serif")
    .attr("font-size", "11px")
    .attr("font-weight", "700")
    .attr("fill", "#FFFFFF")
    .text(d => d.data.icon || "");

  // Main Word / Name Text
  nodeEnter.append("text")
    .attr("class", "card-title")
    .attr("x", 48)
    .attr("y", -8)
    .attr("font-family", "'Sarabun', 'Krub', sans-serif")
    .attr("font-size", d => d.data.type === "pie_root" ? "14.5px" : "13.5px")
    .attr("font-weight", "700")
    .attr("fill", d => (TREE_COLORS[d.data.styleKey] || TREE_COLORS.default).text)
    .text(d => {
      const name = d.data.name || "";
      return name.length > 18 ? name.slice(0, 16) + "..." : name;
    });

  // Subtitle / Detail Text
  nodeEnter.append("text")
    .attr("class", "card-detail")
    .attr("x", 48)
    .attr("y", 11)
    .attr("font-family", "'Sarabun', 'Krub', sans-serif")
    .attr("font-size", "11.5px")
    .attr("fill", "#334E68")
    .text(d => {
      const txt = d.data.detail || d.data.era || "";
      return txt.length > 22 ? txt.slice(0, 20) + "..." : txt;
    });

  // Era or Badge Tag
  nodeEnter.append("text")
    .attr("class", "card-era")
    .attr("x", 48)
    .attr("y", 24)
    .attr("font-family", "'Sarabun', 'Krub', sans-serif")
    .attr("font-size", "10.5px")
    .attr("fill", "#64748B")
    .text(d => d.data.era || "");

  // Expand / Collapse Pill Indicator (at the right edge of cards with children)
  const toggleGroup = nodeEnter.append("g")
    .attr("class", "card-toggle-group")
    .attr("transform", `translate(${cardWidth - 8}, 0)`);

  toggleGroup.append("circle")
    .attr("class", "toggle-circle")
    .attr("r", 9)
    .attr("fill", d => d._children ? (TREE_COLORS[d.data.styleKey] || TREE_COLORS.default).border : "#FFFFFF")
    .attr("stroke", d => (TREE_COLORS[d.data.styleKey] || TREE_COLORS.default).border)
    .attr("stroke-width", 1.4);

  toggleGroup.append("text")
    .attr("class", "toggle-symbol")
    .attr("text-anchor", "middle")
    .attr("dy", 3.5)
    .attr("font-family", "'Sarabun', 'Krub', sans-serif")
    .attr("font-size", "11px")
    .attr("font-weight", "700")
    .attr("fill", d => d._children ? "#FFFFFF" : "#0F2942")
    .text(d => (d.children || d._children) ? (d._children ? "+" : "−") : "");

  // Hide toggle on leaf nodes (no children or _children)
  toggleGroup.style("display", d => (d.children || d._children) ? "block" : "none");

  // Transition nodes to their new position
  const nodeUpdate = node.merge(nodeEnter).transition().duration(400)
    .attr("transform", d => `translate(${d.y},${d.x})`);

  const isDark = document.body.classList.contains('theme-dark');

  nodeUpdate.select(".card-bg")
    .attr("fill", d => (TREE_COLORS[d.data.styleKey] || TREE_COLORS.default).bg)
    .attr("stroke", d => (TREE_COLORS[d.data.styleKey] || TREE_COLORS.default).border)
    .attr("stroke-width", d => (d.data.id === activeWordEntry?.thai_word ? 2.5 : 1.5));

  nodeUpdate.select(".card-title")
    .attr("fill", d => (TREE_COLORS[d.data.styleKey] || TREE_COLORS.default).text);

  nodeUpdate.select(".card-detail")
    .attr("fill", isDark ? "#CBD5E1" : "#334E68");

  nodeUpdate.select(".card-era")
    .attr("fill", isDark ? "#94A3B8" : "#64748B");

  nodeUpdate.select(".toggle-circle")
    .attr("fill", d => d._children ? (TREE_COLORS[d.data.styleKey] || TREE_COLORS.default).border : (isDark ? "#1E293B" : "#FFFFFF"));

  nodeUpdate.select(".toggle-symbol")
    .attr("fill", d => d._children ? "#FFFFFF" : (isDark ? "#F8FAFC" : "#0F2942"))
    .text(d => (d.children || d._children) ? (d._children ? "+" : "−") : "");

  // Transition exiting nodes to the parent's new position
  const nodeExit = node.exit().transition().duration(400)
    .attr("transform", d => `translate(${source.y},${source.x})`)
    .style("opacity", 0)
    .remove();

  // Save current positions for smooth transitions next time
  nodes.forEach(d => {
    d.x0 = d.x;
    d.y0 = d.y;
  });
}

/**
 * Custom Cubic Bezier Horizontal Curve between cards
 */
function diagonalCurve(d, cardWidth) {
  const startX = d.source.y + cardWidth;
  const startY = d.source.x;
  const endX = d.target.y;
  const endY = d.target.x;
  const midX = (startX + endX) / 2;

  return `M ${startX} ${startY}
          C ${midX} ${startY},
            ${midX} ${endY},
            ${endX} ${endY}`;
}

/**
 * Node Click Handler: Toggles expand/collapse and displays detail card
 */
function handleNodeClick(event, d) {
  event.stopPropagation();

  // Toggle children
  if (d.children) {
    d._children = d.children;
    d.children = null;
  } else if (d._children) {
    d.children = d._children;
    d._children = null;
  }

  // Build breadcrumb trail from root to clicked node
  const trail = [];
  let curr = d;
  while (curr) {
    trail.unshift(curr.data.name);
    curr = curr.parent;
  }
  updateBreadcrumb(trail);

  // Select node details in the bottom inspector
  selectNodeDetails(d.data);

  // Re-render
  updateTree(d);
}

/**
 * Updates the top visual breadcrumbs bar
 */
function updateBreadcrumb(trail) {
  const bcEl = document.getElementById("treeBreadcrumb");
  if (!bcEl) return;

  bcEl.innerHTML = trail.map((item, idx) => {
    const isLast = idx === trail.length - 1;
    return `<span class="bc-item ${isLast ? 'bc-active' : ''}">${item}</span>`;
  }).join('<span class="bc-separator"> ➔ </span>');
}

/**
 * Shows rich, friendly detail card below tree when node is clicked
 */
function selectNodeDetails(data) {
  const card = document.getElementById("selectedNodeCard");
  if (!card) return;

  const style = TREE_COLORS[data.styleKey] || TREE_COLORS.default;
  const titleEl = document.getElementById("nodeDetailTitle");
  const badgeEl = document.getElementById("nodeDetailBadge");
  const eraEl = document.getElementById("nodeDetailEra");
  const bodyEl = document.getElementById("nodeDetailBody");
  const tipEl = document.getElementById("nodeDetailTip");

  if (titleEl) titleEl.innerHTML = data.name;
  if (badgeEl) {
    badgeEl.innerText = data.badge || data.type || "ภาษาศาสตร์";
    badgeEl.style.color = style.text;
    badgeEl.style.borderColor = style.border;
  }
  if (eraEl) eraEl.innerText = data.era || "ช่วงเวลาประวัติศาสตร์";

  let explanation = "";
  let mnemonic = "";

  if (data.type === "pie_root") {
    explanation = `<strong>รากภาษาบรรพบุรุษอินโด-ยูโรเปียน (PIE)</strong> คือต้นกำเนิดของภาษากว่า 400 ภาษาในโลก ทั้งในยุโรป อิหร่าน และอินเดีย โดยมีอายุเก่าแก่กว่า 6,000 ปี`;
    mnemonic = `ความหมายดั้งเดิมคือ "${data.detail}" ซึ่งแตกสาขาออกไปเป็นทั้งคำไทย (ผ่านบาลี-สันสกฤต) และคำอังกฤษ`;
  } else if (data.type === "thai_leaf") {
    explanation = `<strong>คำในภาษาไทย:</strong> ${data.subDetail || data.detail}<br>เป็นคำที่เรารู้จักและใช้ในชีวิตประจำวัน โดยรับถ่ายทอดผ่านพระพุทธศาสนาและวรรณคดีอินเดียโบราณ`;
    mnemonic = `คำไทยคำนี้มีคำร่วมเชื้อสายซ่อนอยู่ในภาษาอังกฤษ ลองคลิกดูสายเจอร์แมนิกและละติน`;
  } else if (data.type === "intermediate_lang") {
    explanation = `<strong>ภาษาสันสกฤต/บาลี (Indic):</strong> ภาษาศักดิ์สิทธิ์โบราณแห่งลุ่มแม่น้ำสินทุ ${data.detail}`;
    mnemonic = `คำสันสกฤตนี้คือสะพานเชื่อมสำคัญที่ส่งคำต่อไปยังภาษาไทยในยุคสุโขทัยและอยุธยา`;
  } else if (data.type === "english_leaf") {
    explanation = `<strong>คำภาษาอังกฤษ:</strong> ${data.detail} (${data.era})<br>วิวัฒนาการ: ${data.subDetail || 'สืบทอดตามสายอินโด-ยูโรเปียน'}`;
    mnemonic = `<strong>เทคนิคช่วยจำ (Mnemonic):</strong> ${data.note || 'มีความหมายและรากศัพท์ร่วมกันกับคำไทย'}`;
  } else {
    explanation = `<strong>${data.name}:</strong> ${data.detail || 'สาขาตระกูลภาษาที่แยกตัวตามการอพยพของผู้คนในยุคโบราณ'}`;
    mnemonic = `มีคำศัพท์ในกิ่งนี้ ${data.badge || ''} คลิกที่เครื่องหมาย [+] เพื่อเปิดดู`;
  }

  if (bodyEl) bodyEl.innerHTML = explanation;
  if (tipEl) tipEl.innerHTML = mnemonic;
  card.style.display = "block";
}

/**
 * Expands all tree nodes
 */
function expandAllNodes() {
  if (!treeRoot) return;
  function expand(d) {
    if (d._children) {
      d.children = d._children;
      d._children = null;
    }
    if (d.children) d.children.forEach(expand);
  }
  expand(treeRoot);
  updateTree(treeRoot);
  resetTreeView();
}

/**
 * Collapses nodes to only primary branches
 */
function collapseAllNodes() {
  if (!treeRoot) return;
  function collapse(d) {
    if (d.children) {
      d._children = d.children;
      d._children.forEach(collapse);
      d.children = null;
    }
  }
  // Keep first level children visible, collapse deeper
  if (treeRoot.children) {
    treeRoot.children.forEach(collapse);
  }
  updateTree(treeRoot);
  resetTreeView();
}

/**
 * Resets pan and zoom to center
 */
function resetTreeView() {
  if (!treeSvg || !treeZoom) return;
  const svgNode = treeSvg.node();
  const width = svgNode ? (svgNode.getBoundingClientRect().width || 960) : 960;
  const initialTransform = d3.zoomIdentity.translate(60, 260).scale(0.85);

  treeSvg.transition().duration(500).call(treeZoom.transform, initialTransform);
}

/**
 * Toggles between D3 Graphic Tree and Cascading Card Flow
 */
let isGraphicView = true;
function toggleGraphicOrCardView() {
  const treeCard = document.getElementById("d3GraphicWrapper");
  const flowCard = document.getElementById("cascadingFlowWrapper");
  const btn = document.getElementById("graphToggleViewBtn");

  isGraphicView = !isGraphicView;
  if (isGraphicView) {
    if (treeCard) treeCard.style.display = "block";
    if (flowCard) flowCard.style.display = "none";
    if (btn) btn.innerText = "ดูแบบการ์ดลำดับชั้น";
  } else {
    if (treeCard) treeCard.style.display = "none";
    if (flowCard) flowCard.style.display = "block";
    if (btn) btn.innerText = "ดูแบบผังต้นไม้กราฟิก";
  }
}

/**
 * Renders the Step-by-Step Cascading Card View (Mobile & Document Friendly)
 */
function renderCascadingCardView(rootData) {
  const container = document.getElementById("cascadingFlowContainer");
  if (!container || !rootData) return;

  const eastern = rootData.children?.[0];
  const western = rootData.children?.[1];

  let html = `
    <!-- Root PIE Card -->
    <div class="cascade-root-card">
      <div class="cascade-header">
        <span class="cascade-icon">PIE</span>
        <div>
          <span class="cascade-tag">รากบรรพบุรุษร่วม (PIE Root)</span>
          <h3 class="cascade-title">${rootData.name}</h3>
        </div>
        <span class="cascade-era">${rootData.era}</span>
      </div>
      <p class="cascade-desc">${rootData.detail} • ${rootData.subDetail}</p>
    </div>

    <div class="cascade-branches-grid">
      <!-- Eastern Branch: Towards Thai -->
      <div class="cascade-branch-col eastern-col">
        <div class="branch-col-header">
          <span class="col-icon">SAN</span>
          <div>
            <h4 class="col-title">${eastern?.name || 'สายตะวันออก'}</h4>
            <span class="col-subtitle">${eastern?.detail || 'สายอินโด-อารยัน สู่ภาษาไทย'}</span>
          </div>
        </div>

        <div class="branch-cards-stack">
          ${(eastern?.children || []).map(indic => `
            <div class="cascade-step-card">
              <div class="step-tag indic-tag">IND: ${indic.badge} (${indic.era})</div>
              <div class="step-word">${indic.name}</div>
              <div class="step-sub">${indic.detail}</div>
              <div class="step-arrow">➔ ถ่ายทอดสู่ภาษาไทย</div>
              
              ${(indic.children || []).map(th => `
                <div class="cascade-target-card thai-target">
                  <div class="target-tag">TH: ภาษาไทย (${th.era})</div>
                  <div class="target-word">${th.name} <span class="target-phonetic">${th.detail}</span></div>
                  <p class="target-def">${th.subDetail}</p>
                </div>
              `).join('')}
            </div>
          `).join('')}
        </div>
      </div>

      <!-- Western Branch: Towards English -->
      <div class="cascade-branch-col western-col">
        <div class="branch-col-header">
          <span class="col-icon">EUR</span>
          <div>
            <h4 class="col-title">${western?.name || 'สายตะวันตก'}</h4>
            <span class="col-subtitle">${western?.badge || 'สายยุโรป สู่ภาษาอังกฤษ'}</span>
          </div>
        </div>

        <div class="western-subfamilies-list">
          ${(western?.children || []).map(sub => `
            <div class="subfamily-group">
              <div class="subfamily-header">
                <span class="sub-icon">${sub.icon}</span>
                <span class="sub-title">${sub.name}</span>
                <span class="sub-badge">${sub.badge}</span>
              </div>
              <div class="subfamily-words-grid">
                ${(sub.children || []).map(en => `
                  <div class="en-cognate-pill-card" onclick="selectNodeDetails(${JSON.stringify(en).replace(/"/g, '&quot;')})">
                    <div class="en-pill-top">
                      <span class="en-pill-word">${en.name}</span>
                      <span class="en-pill-diff">${en.badge}</span>
                    </div>
                    <div class="en-pill-origin">${en.detail}</div>
                    <div class="en-pill-note">${en.note || en.subDetail}</div>
                  </div>
                `).join('')}
              </div>
            </div>
          `).join('')}
        </div>
      </div>
    </div>
  `;

  container.innerHTML = html;
}

// Global exports
window.renderD3Graph = renderD3Graph;
window.updateTree = updateTree;

