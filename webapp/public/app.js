/**
 * Kinetic Typography & Liquid Glass UI Controller
 * Core Philosophy: Kinetic Minimalism
 *
 * Requirements:
 * 1. Ambient State (เฟสแรก):
 *    - คำศัพท์เยอะและหนาแน่น ("รกกว่านี้") วิ่งไปมาทั้งแนวตั้งและแนวนอน
 *    - คำทั้ง 5 ("วันนี้", "อยากให้", "ภาษาไทย", "ทำอะไร", "ให้คุณ") ปะปนอยู่ในกลุ่มคำที่วิ่งอย่างเป็นธรรมชาติ
 *    - ตรงกลางเปิดโล่ง ไม่มีตัวหนังสือบอกล่วงหน้า
 * 2. Focus / Click:
 *    - "คำที่ขึ้นคือการประกอบกันของคำที่วิ่ง ไม่ใช่เสกขึ้นมา"
 *    - คำทั้ง 5 วิ่งจากจุดที่กำลังลอยอยู่จริง บินมารวมตัว จัดแถวตรงกลาง ขยายใหญ่เป็นประโยคหลัก
 *    - คำสุดท้ายที่เรียงแล้ว: "ตัวอักษรมีหัว หนา ชัด" (Sarabun Bold 800)
 * 3. Background:
 *    - "ข้างหลังไม่ได้หายไปเลย แต่วิ่งช้าลงและค่อยๆ จางลง"
 *    - คำข้างหลังค่อยๆ ลดความเร็วลง (Deceleration) และค่อยๆ จางลงอย่างนุ่มนวล ยังคงลอยอยู่เป็นฉากหลัง
 */

document.addEventListener('DOMContentLoaded', () => {
  // DOM Elements
  const body = document.body;
  const searchInput = document.getElementById('thaiSearchInput');
  const actionSubmitBtn = document.getElementById('actionSubmitBtn');
  const greetingHeadline = document.getElementById('greetingHeadline');
  const greetingSubtext = document.getElementById('greetingSubtext');
  const wordsCloud = document.getElementById('wordsCloud');
  const statusLabel = document.getElementById('statusLabel');
  const suggestionTray = document.getElementById('suggestionTray');
  const responseDrawer = document.getElementById('responseDrawer');
  const responseBody = document.getElementById('responseBody');
  const closeResponseBtn = document.getElementById('closeResponseBtn');

  // Current State: 'ambient' | 'aligned' | 'active'
  let currentState = 'ambient';

  // 180+ Thai Words Bank (dense & rich vocabulary for Phase 1)
  const denseWordList = [
    // The 5 target words that will assemble into the headline
    { text: "วันนี้", size: 18, weight: "regular", isTarget: true, targetIndex: 0, startXRatio: 0.16, startYRatio: 0.24 },
    { text: "อยากให้", size: 18, weight: "regular", isTarget: true, targetIndex: 1, startXRatio: 0.78, startYRatio: 0.18 },
    { text: "ภาษาไทย", size: 19, weight: "regular", isTarget: true, targetIndex: 2, startXRatio: 0.12, startYRatio: 0.68 },
    { text: "ทำอะไร", size: 18, weight: "regular", isTarget: true, targetIndex: 3, startXRatio: 0.84, startYRatio: 0.64 },
    { text: "ให้คุณ", size: 18, weight: "regular", isTarget: true, targetIndex: 4, startXRatio: 0.52, startYRatio: 0.12 },

    // Dense background vocabulary (Linguistics, society, technology, dreams, culture)
    { text: "ความรู้", size: 22, weight: "bold" },
    { text: "อนาคต", size: 17, weight: "regular" },
    { text: "การศึกษา", size: 17, weight: "regular" },
    { text: "เพื่อน", size: 15, weight: "light" },
    { text: "พัฒนา", size: 16, weight: "medium" },
    { text: "แนวทาง", size: 16, weight: "regular" },
    { text: "โอกาส", size: 18, weight: "regular" },
    { text: "เทคโนโลยี", size: 20, weight: "bold" },
    { text: "ความสำเร็จ", size: 20, weight: "bold" },
    { text: "เข้าใจ", size: 17, weight: "regular" },
    { text: "ศักยภาพ", size: 18, weight: "medium" },
    { text: "แรงบันดาลใจ", size: 19, weight: "bold" },
    { text: "สร้างสรรค์", size: 19, weight: "bold" },
    { text: "เป้าหมาย", size: 16, weight: "light" },
    { text: "สุข", size: 18, weight: "bold" },
    { text: "ฝึกฝน", size: 16, weight: "regular" },
    { text: "สังคม", size: 17, weight: "medium" },
    { text: "ครอบครัว", size: 15, weight: "light" },
    { text: "ประโยชน์", size: 16, weight: "regular" },
    { text: "ไอเดีย", size: 17, weight: "medium" },
    { text: "นวัตกรรม", size: 16, weight: "light" },
    { text: "ทะเยอทะยาน", size: 18, weight: "medium" },
    { text: "ข้อความ", size: 19, weight: "bold" },
    { text: "ประเทศไทย", size: 19, weight: "bold" },
    { text: "สิ่งแวดล้อม", size: 16, weight: "regular" },
    { text: "ความฝัน", size: 23, weight: "bold" },
    { text: "เสรีภาพ", size: 17, weight: "regular" },
    { text: "การใช้ชีวิต", size: 16, weight: "regular" },
    { text: "ความสุข", size: 22, weight: "bold" },
    { text: "มิตรภาพ", size: 19, weight: "bold" },
    { text: "คุณ", size: 19, weight: "bold" },
    { text: "สิ่งที่ชอบ", size: 18, weight: "bold" },
    { text: "ความยั่งยืน", size: 21, weight: "bold" },
    { text: "เวลา", size: 19, weight: "medium" },
    { text: "คุณภาพชีวิต", size: 16, weight: "regular" },
    { text: "การตลาด", size: 17, weight: "regular" },
    { text: "ธุรกิจ", size: 16, weight: "regular" },
    { text: "ความคิดสร้างสรรค์", size: 16, weight: "light" },
    { text: "เทคโนโลยี AI", size: 17, weight: "medium" },
    { text: "แก้ปัญหา", size: 16, weight: "light" },
    { text: "สถิติ", size: 15, weight: "light" },
    { text: "การวางแผน", size: 16, weight: "light" },
    { text: "มุมมอง", size: 15, weight: "light" },
    { text: "ใจ", size: 20, weight: "bold" },
    { text: "การเติบโต", size: 17, weight: "regular" },
    { text: "โอกาสใหม่", size: 15, weight: "light" },
    { text: "วิจัย", size: 15, weight: "light" },
    { text: "เส้นทาง", size: 16, weight: "light" },
    { text: "ภาษาศาสตร์", size: 19, weight: "medium" },
    { text: "รากศัพท์", size: 20, weight: "bold" },
    { text: "สุนทรียภาพ", size: 18, weight: "regular" },
    { text: "วรรณศิลป์", size: 17, weight: "regular" },
    { text: "คำร่วมเชื้อสาย", size: 19, weight: "bold" },
    { text: "ปิยวาจา", size: 16, weight: "light" },
    { text: "อรรถรส", size: 16, weight: "light" },
    { text: "จิตวิญญาณ", size: 18, weight: "medium" },
    { text: "มรดกทางภาษา", size: 18, weight: "regular" },
    { text: "ปัญญาประดิษฐ์", size: 19, weight: "bold" },
    { text: "ความเข้าใจ", size: 17, weight: "regular" },
    { text: "สายสัมพันธ์", size: 17, weight: "regular" },
    { text: "ธรรมชาติ", size: 17, weight: "medium" },
    { text: "มนุษยชาติ", size: 18, weight: "medium" },
    { text: "สันติภาพ", size: 18, weight: "bold" },
    { text: "ความหวัง", size: 21, weight: "bold" },
    { text: "จินตนาการ", size: 19, weight: "bold" },
    { text: "พลังใจ", size: 17, weight: "medium" },
    { text: "ความทรงจำ", size: 17, weight: "light" },
    { text: "การเปลี่ยนแปลง", size: 17, weight: "regular" },
    { text: "การเดินทาง", size: 16, weight: "light" },
    { text: "ศิลปะ", size: 18, weight: "medium" },
    { text: "ปรัชญา", size: 18, weight: "bold" },
    { text: "สัทศาสตร์", size: 17, weight: "regular" },
    { text: "วากยสัมพันธ์", size: 16, weight: "light" },
    { text: "อักขระ", size: 18, weight: "bold" },
    { text: "ลายสือไทย", size: 18, weight: "medium" },
    { text: "มรดก", size: 17, weight: "regular" },
    { text: "ความรัก", size: 21, weight: "bold" },
    { text: "คุณค่า", size: 18, weight: "medium" },
    { text: "ความเมตตา", size: 17, weight: "light" },
    { text: "ความสงบ", size: 17, weight: "light" },
    { text: "วิสัยทัศน์", size: 18, weight: "bold" },
    { text: "โลกทัศน์", size: 17, weight: "regular" },
    { text: "การแบ่งปัน", size: 16, weight: "light" },
    { text: "ความเพียร", size: 17, weight: "medium" },
    { text: "ความลึกซึ้ง", size: 18, weight: "medium" },
    { text: "ความสดใส", size: 17, weight: "light" },
    { text: "ความกล้าหาญ", size: 18, weight: "bold" },
    { text: "ความสามัคคี", size: 18, weight: "medium" },
    { text: "ความภูมิใจ", size: 18, weight: "medium" },
    { text: "ความมั่นคง", size: 18, weight: "regular" },
    { text: "ความเจริญ", size: 18, weight: "medium" },
    { text: "ความรุ่งโรจน์", size: 19, weight: "bold" },
    { text: "บาลี", size: 17, weight: "regular" },
    { text: "สันสกฤต", size: 18, weight: "bold" },
    { text: "Proto-Indo-European", size: 15, weight: "light" },
    { text: "ไวยากรณ์", size: 16, weight: "light" },
    { text: "พจนานุกรม", size: 19, weight: "bold" },
    { text: "อรรถศาสตร์", size: 16, weight: "light" },
    { text: "กวีนิพนธ์", size: 17, weight: "medium" },
    { text: "ร้อยกรอง", size: 16, weight: "light" },
    { text: "โวหาร", size: 16, weight: "light" },
    { text: "คำไวพจน์", size: 17, weight: "medium" },
    { text: "คำซ้อน", size: 15, weight: "light" },
    { text: "คำสมาส", size: 16, weight: "light" },
    { text: "คำสนธิ", size: 16, weight: "light" },
    { text: "สัมผัส", size: 16, weight: "regular" },
    { text: "บริบท", size: 16, weight: "light" },
    { text: "วิวัฒนาการ", size: 18, weight: "medium" },
    { text: "คำสืบทอด", size: 17, weight: "light" },
    { text: "ตระกูลภาษา", size: 17, weight: "medium" },
    { text: "การแผลงคำ", size: 16, weight: "light" },
    { text: "เสียงวรรณยุกต์", size: 16, weight: "light" },
    { text: "อักษรนำ", size: 15, weight: "light" },
    { text: "ตัวสะกด", size: 15, weight: "light" },
    { text: "สระสนธิ", size: 16, weight: "light" },
    { text: "พยัญชนะ", size: 17, weight: "regular" },
    { text: "ศัพทมูลวิทยา", size: 17, weight: "bold" },
    { text: "รากศัพท์ดั้งเดิม", size: 17, weight: "medium" },
    { text: "คำมูล", size: 16, weight: "light" },
    { text: "สัทอักษร", size: 16, weight: "light" },
    { text: "ภาษาถิ่น", size: 16, weight: "light" },
    { text: "ภาษามาตรฐาน", size: 17, weight: "regular" },
    { text: "คลังคำ", size: 18, weight: "bold" },
    { text: "ปัญญาญาน", size: 17, weight: "medium" },
    { text: "มิติภาษา", size: 17, weight: "light" },
    { text: "คลังความรู้", size: 19, weight: "bold" },
    { text: "การอนุรักษ์", size: 16, weight: "light" },
    { text: "สถาบันภาษา", size: 17, weight: "medium" },
    { text: "วิชาการ", size: 16, weight: "regular" },
    { text: "การรังสรรค์", size: 17, weight: "medium" },
    { text: "จารึก", size: 17, weight: "bold" },
    { text: "ศิลาจารึก", size: 17, weight: "medium" },
    { text: "สุโขทัย", size: 16, weight: "light" },
    { text: "อยุธยา", size: 16, weight: "light" },
    { text: "รัตนโกสินทร์", size: 17, weight: "regular" },
    { text: "อารยธรรมเอเชีย", size: 16, weight: "light" },
    { text: "มรดกโลก", size: 18, weight: "bold" },
    { text: "วิถีชีวิต", size: 16, weight: "regular" },
    { text: "ความงดงาม", size: 18, weight: "medium" },
    { text: "ความอ่อนหวาน", size: 16, weight: "light" },
    { text: "ความไพเราะ", size: 17, weight: "medium" },
    { text: "เสียงดนตรี", size: 16, weight: "light" },
    { text: "ท่วงทำนอง", size: 16, weight: "light" },
    { text: "จังหวะ", size: 16, weight: "light" },
    { text: "การออกเสียง", size: 16, weight: "regular" },
    { text: "อรรถาธิบาย", size: 16, weight: "light" },
    { text: "ความกระจ่าง", size: 17, weight: "regular" },
    { text: "การขยายขอบเขต", size: 16, weight: "light" },
    { text: "ไร้พรมแดน", size: 17, weight: "medium" },
    { text: "อนาคตกาล", size: 16, weight: "light" },
    { text: "ปัจจุบันกาล", size: 16, weight: "light" },
    { text: "อดีตกาล", size: 16, weight: "light" },
    { text: "กาลเวลา", size: 18, weight: "medium" },
    { text: "ความเชื่อมโยง", size: 18, weight: "bold" },
    { text: "ความผูกพัน", size: 17, weight: "light" },
    { text: "สะพานเชื่อม", size: 19, weight: "bold" },
    { text: "ประตูความรู้", size: 18, weight: "bold" },
    { text: "แสงสว่าง", size: 18, weight: "bold" },
    { text: "ประกายความคิด", size: 17, weight: "medium" },
    { text: "ดวงตะวัน", size: 16, weight: "light" },
    { text: "ดวงจันทร์", size: 16, weight: "light" },
    { text: "ท้องฟ้า", size: 16, weight: "light" },
    { text: "ผืนดิน", size: 16, weight: "light" },
    { text: "สายน้ำ", size: 16, weight: "light" },
    { text: "สายลม", size: 16, weight: "light" },
    { text: "ความอบอุ่นใจ", size: 16, weight: "light" },
    { text: "ความมีชีวิตชีวา", size: 17, weight: "medium" },
    { text: "พลังแห่งถ้อยคำ", size: 19, weight: "bold" },
    { text: "ศิลปวัฒนธรรม", size: 18, weight: "medium" },
    { text: "ภูมิปัญญาไทย", size: 20, weight: "bold" },
    { text: "ราชบัณฑิตยสภา", size: 20, weight: "bold" }
  ];

  // Particle tracking
  const wordParticles = [];
  const targetParticles = []; // The 5 target word tokens that assemble into the headline
  let currentSpeedMultiplier = 1.0;
  let targetSpeedMultiplier = 1.0;
  let isAssembled = false;

  // Initialize Dense Words Cloud
  function initWordsCloud() {
    wordsCloud.innerHTML = '';
    wordParticles.length = 0;
    targetParticles.length = 0;

    const screenWidth = window.innerWidth;
    const screenHeight = window.innerHeight;

    denseWordList.forEach((item, index) => {
      const el = document.createElement('div');
      el.className = `word-token weight-${item.weight}`;
      el.textContent = item.text;
      el.style.fontSize = `${item.size}px`;

      // Assign position across screen
      let posX = Math.random() * (screenWidth - 120);
      let posY = Math.random() * (screenHeight - 80);

      if (item.startXRatio !== undefined) {
        posX = item.startXRatio * screenWidth;
        posY = item.startYRatio * screenHeight;
      }

      // High kinetic velocities in both horizontal and vertical directions
      const dir = index % 4;
      let vx = 0;
      let vy = 0;

      if (dir === 0) {
        // Horizontal left/right
        vx = (Math.random() > 0.5 ? 1 : -1) * (0.65 + Math.random() * 0.9);
        vy = (Math.random() - 0.5) * 0.35;
      } else if (dir === 1) {
        // Vertical up/down (วิ่งแนวตั้งเยอะๆ)
        vx = (Math.random() - 0.5) * 0.35;
        vy = (Math.random() > 0.5 ? 1 : -1) * (0.60 + Math.random() * 0.85);
      } else if (dir === 2) {
        // Diagonal
        vx = (Math.random() > 0.5 ? 1 : -1) * (0.50 + Math.random() * 0.65);
        vy = (Math.random() > 0.5 ? 1 : -1) * (0.50 + Math.random() * 0.65);
      } else {
        // Slanted horizontal
        vx = (Math.random() > 0.5 ? 1 : -1) * (0.75 + Math.random() * 0.7);
        vy = (Math.random() - 0.5) * 0.45;
      }

      const baseOpacity = item.weight === 'bold' ? 0.35 : (item.weight === 'medium' ? 0.28 : 0.20);
      el.dataset.baseOpacity = baseOpacity;
      el.style.opacity = baseOpacity;

      wordsCloud.appendChild(el);

      const pObj = {
        el,
        x: posX,
        y: posY,
        vx,
        vy,
        baseOpacity,
        size: item.size,
        weight: item.weight,
        isTarget: !!item.isTarget,
        targetIndex: item.targetIndex,
        isAssembling: false
      };

      wordParticles.push(pObj);

      if (item.isTarget) {
        targetParticles[item.targetIndex] = pObj;
      }
    });

    populateStreams();
  }

  // Populate Horizontal and Vertical Streams
  function populateStreams() {
    const hPhrases = [
      "ความรู้ • ภาษาศาสตร์ • รากศัพท์สันสกฤต • คำร่วมเชื้อสาย • นวัตกรรม • วัฒนธรรม • ศัพทานุกรม",
      "Proto-Indo-European • วิวัฒนาการเสียง • บาลี • ภาษาไทย • ภูมิปัญญา • ความหมาย • คลังความรู้",
      "มรดกทางภาษา • อักขระวิธี • รากศัพท์โบราณ • การเรียนรู้แห่งอนาคต • จินตนาการ • ปัญญาประดิษฐ์",
      "สุนทรียศาสตร์ • อรรถศาสตร์ • ไวยากรณ์ • เสียงสัมผัส • ร้อยกรอง • อักษรไทย • พจนานุกรมร่วมสมัย",
      "คำยืม • ปิยวาจา • อารยธรรม • ความรุ่งเรือง • ศักยภาพ • ความคิดสร้างสรรค์ • สันติสุข"
    ];

    const vPhrases = [
      "ภาษาไทย • วิทยาศาสตร์ • ศิลปวัฒนธรรม • ภูมิปัญญาแผ่นดิน • จารึกประวัติศาสตร์",
      "รากศัพท์ • พจนานุกรม • สัทศาสตร์ • วากยสัมพันธ์ • ศาสตร์แห่งเสียง",
      "ความงดงาม • ตัวอักษรไทย • ลายสือไทย • สุนทรียภาพแห่งภาษา",
      "ความคิดสร้างสรรค์ • การสื่อสาร • นวัตกรรมร่วมสมัย • ความยั่งยืน",
      "ปัญญารู้คิด • มนุษยศาสตร์ • การเชื่อมโยงโลก • สันติศึกษา"
    ];

    for (let i = 1; i <= 5; i++) {
      const hEl = document.getElementById(`streamH${i}`);
      if (hEl) {
        const text = hPhrases[(i - 1) % hPhrases.length];
        hEl.textContent = `${text} • ${text} • ${text}`;
      }

      const vEl = document.getElementById(`streamV${i}`);
      if (vEl) {
        const text = vPhrases[(i - 1) % vPhrases.length];
        vEl.textContent = `${text} • ${text} • ${text}`;
      }
    }
  }

  // Animation Loop: Updates all moving words
  function animateParticles() {
    currentSpeedMultiplier += (targetSpeedMultiplier - currentSpeedMultiplier) * 0.05;

    const screenWidth = window.innerWidth;
    const screenHeight = window.innerHeight;

    for (let i = 0; i < wordParticles.length; i++) {
      const p = wordParticles[i];

      // If this target word is currently assembling/locked into the headline, skip free physics
      if (p.isAssembling) continue;

      p.x += p.vx * currentSpeedMultiplier;
      p.y += p.vy * currentSpeedMultiplier;

      // Wrap around screen edges
      if (p.x < -140) p.x = screenWidth + 20;
      else if (p.x > screenWidth + 20) p.x = -140;

      if (p.y < -70) p.y = screenHeight + 20;
      else if (p.y > screenHeight + 20) p.y = -70;

      p.el.style.transform = `translate3d(${p.x}px, ${p.y}px, 0)`;
    }

    requestAnimationFrame(animateParticles);
  }

  // State Management Engine
  function setState(state) {
    currentState = state;
    body.classList.remove('state-ambient', 'state-aligned', 'state-active');
    body.classList.add(`state-${state}`);

    if (state === 'ambient') {
      targetSpeedMultiplier = 1.0; // วิ่งเต็มสปีดใน Ambient
      disperseToAmbient();
    } else if (state === 'aligned') {
      targetSpeedMultiplier = 0.22; // ชะลอช้าลงนุ่มนวล
      triggerAssembly();
    } else if (state === 'active') {
      targetSpeedMultiplier = 0.18; // ชะลอช้าลงนุ่มนวล เป็นพื้นหลัง
      triggerAssembly();
    }
  }

  // Trigger Genuine Visual Assembly:
  // "คำที่ขึ้นคือการประกอบกันของคำที่วิ่งไม่ใช่เสกขึ้นมา เเละข้างหลังก็ไม่ได้หายไปเลยเเต่วิ่งช้าลงเเละค่อย ๆ จางลง"
  function triggerAssembly() {
    if (isAssembled) return;
    isAssembled = true;

    const isDark = body.classList.contains('theme-dark');
    const dimOpacity = isDark ? 0.18 : 0.13;

    // 1. Fade other background words softly so they remain clearly visible as they decelerate
    wordParticles.forEach(p => {
      if (!p.isTarget) {
        p.el.style.opacity = dimOpacity;
        p.el.style.filter = 'blur(0.6px)';
      }
    });

    // 2. Measure target headline font size and anchor slots
    greetingHeadline.style.opacity = '1';
    greetingSubtext.style.opacity = '0.85';
    greetingSubtext.style.transform = 'translateY(0)';

    // Compute headline target font size
    const computedFontSize = window.getComputedStyle(greetingHeadline).fontSize;

    // 3. For each of the 5 running target words: Glide from live position straight to slot!
    targetParticles.forEach((p, idx) => {
      if (!p) return;
      p.isAssembling = true;
      p.el.classList.remove('is-docked-hidden');
      p.el.style.display = '';
      p.el.classList.add('is-target-assembler');

      const targetSlot = document.getElementById(`targetSlot${idx}`);
      if (!targetSlot) return;

      const slotRect = targetSlot.getBoundingClientRect();
      const destX = slotRect.left;
      const destY = slotRect.top;

      // Smooth cubic-bezier flight directly into position
      p.el.style.transition = `
        transform 1.25s cubic-bezier(0.16, 1, 0.3, 1) ${idx * 40}ms,
        font-size 1.25s cubic-bezier(0.16, 1, 0.3, 1) ${idx * 40}ms,
        color 0.6s ease,
        opacity 0.6s ease
      `;

      p.el.style.transform = `translate3d(${destX}px, ${destY}px, 0)`;
      p.el.style.fontSize = computedFontSize;
      p.el.style.fontFamily = "'Sarabun', 'Krub', sans-serif";
      p.el.style.fontWeight = '800'; // หนา ชัด มีหัว
      p.el.style.color = isDark ? '#FFFFFF' : '#0F2942';
      p.el.style.opacity = '1';
      p.el.style.filter = 'none';
      p.el.style.textShadow = isDark ? '0 0 25px rgba(56, 189, 248, 0.35)' : '0 1px 2px rgba(0, 0, 0, 0.15)';
    });

    // 4. Dock words into genuine DOM slots
    setTimeout(() => {
      greetingHeadline.classList.add('is-docked');
      // Reveal genuine slots in DOM flow so they scroll naturally
      document.querySelectorAll('.greeting-slot').forEach(slot => {
        slot.style.visibility = 'visible';
      });
      // Hide the floating particles so they don't duplicate
      targetParticles.forEach(p => {
        if (p && p.el) {
          p.el.classList.remove('is-target-assembler');
          p.el.classList.add('is-docked-hidden');
          p.el.style.display = 'none';
          p.el.style.opacity = '0';
        }
      });
    }, 1300);
  }

  // Disperse back to Ambient State
  function disperseToAmbient() {
    isAssembled = false;

    // Reset headline visibility and docked status
    greetingHeadline.style.opacity = '0';
    greetingHeadline.classList.remove('is-docked');
    document.querySelectorAll('.greeting-slot').forEach(slot => {
      slot.style.visibility = 'hidden';
    });
    greetingSubtext.style.opacity = '0';
    greetingSubtext.style.transform = 'translateY(8px)';

    const screenWidth = window.innerWidth;
    const screenHeight = window.innerHeight;

    // Return the 5 target words back to the cloud as regular floating words
    targetParticles.forEach((p, idx) => {
      if (!p) return;
      p.isAssembling = false;
      p.el.classList.remove('is-target-assembler', 'is-docked-hidden');
      p.el.style.display = '';
      p.el.style.opacity = p.baseOpacity;

      // Random position in different screen quadrants
      const itemConfig = denseWordList[idx];
      p.x = (itemConfig.startXRatio || Math.random()) * (screenWidth - 120);
      p.y = (itemConfig.startYRatio || Math.random()) * (screenHeight - 80);

      p.el.style.transition = 'none';
      p.el.style.transform = `translate3d(${p.x}px, ${p.y}px, 0)`;
      p.el.style.fontSize = `${p.size}px`;
      p.el.style.fontFamily = '';
      p.el.style.fontWeight = '400';
      p.el.style.color = '';
      p.el.style.opacity = p.baseOpacity;
      p.el.style.filter = 'none';
    });

    // Restore all background words to full opacity and crispness
    wordParticles.forEach(p => {
      p.el.style.opacity = p.baseOpacity;
      p.el.style.filter = 'none';
    });
  }

  // Mode Selection Elements
  const modeSelectionContainer = document.getElementById('modeSelectionContainer');
  const typedWordLabel = document.getElementById('typedWordLabel');
  const echoWordSpans = document.querySelectorAll('.echo-word');
  const modeCardGeneral = document.getElementById('modeCardGeneral');
  const modeCardEtymology = document.getElementById('modeCardEtymology');
  const responseStatusText = document.getElementById('responseStatusText');

  // Focus & Click Events on Textbox
  searchInput.addEventListener('focus', () => {
    const value = searchInput.value.trim();
    if (value.length > 0) {
      setState('active');
      showModeSelection(value);
    } else {
      setState('aligned');
    }
  });

  // ขณะพิมพ์: โหลดโหมดให้เลือก 2 ก้อนใต้ช่องพิมพ์
  searchInput.addEventListener('input', (e) => {
    const value = e.target.value.trim();
    if (value.length > 0) {
      setState('active');
      actionSubmitBtn.classList.add('is-active');
      actionSubmitBtn.disabled = false;

      // โหลด 2 ก้อนตัวเลือกใต้ช่องพิมพ์ พร้อมใส่คำที่กำลังพิมพ์ลงไปในพรีวิว
      showModeSelection(value);
    } else {
      setState('aligned');
      actionSubmitBtn.classList.remove('is-active');
      actionSubmitBtn.disabled = true;

      // ซ่อนโหมด และซ่อนคำตอบเมื่อลบคำออก
      hideModeSelection();
    }
  });

  // แสดง 2 ก้อนโหมดใต้ช่องพิมพ์
  function showModeSelection(word) {
    typedWordLabel.textContent = word;
    echoWordSpans.forEach(span => {
      span.textContent = word;
    });

    modeSelectionContainer.style.display = 'block';
    if (suggestionTray) suggestionTray.style.display = 'none';
  }

  // ซ่อน 2 ก้อนโหมด
  function hideModeSelection() {
    modeSelectionContainer.style.display = 'none';
    if (suggestionTray) suggestionTray.style.display = 'flex';
    responseDrawer.classList.remove('show');
    modeCardGeneral.classList.remove('is-selected');
    modeCardEtymology.classList.remove('is-selected');
  }

  // คลิกเลือกก้อนที่ 1: ค้นหาทั่วไป (General Search - WACHA)
  modeCardGeneral.addEventListener('click', () => {
    const query = searchInput.value.trim();
    if (!query) return;

    modeCardGeneral.classList.add('is-selected');
    modeCardEtymology.classList.remove('is-selected');
    executeGeneralSearch(query);
  });

  // คลิกเลือกก้อนที่ 2: การหารากศัพท์ (Etymological Bridge)
  modeCardEtymology.addEventListener('click', () => {
    const query = searchInput.value.trim();
    if (!query) return;

    modeCardEtymology.classList.add('is-selected');
    modeCardGeneral.classList.remove('is-selected');
    executeEtymologySearch(query);
  });

  // Clicking anywhere on the liquid textbox triggers focus
  document.getElementById('liquidTextbox').addEventListener('click', () => {
    searchInput.focus();
  });

  // Click outside to smoothly return to Ambient State if input is empty
  document.addEventListener('click', (e) => {
    if (currentState === 'aligned' && !searchInput.value.trim()) {
      const isInsideTextbox = e.target.closest('#liquidTextbox');
      const isInsideModes = e.target.closest('#modeSelectionContainer');
      const isInsideTheme = e.target.closest('#themeToggleBtn');
      if (!isInsideTextbox && !isInsideModes && !isInsideTheme) {
        searchInput.blur();
        setState('ambient');
      }
    }
  });

  // Keyboard Shortcuts (Enter & Escape)
  searchInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && searchInput.value.trim().length > 0) {
      e.preventDefault();
      // Default to General search on Enter
      modeCardGeneral.classList.add('is-selected');
      executeGeneralSearch(searchInput.value.trim());
    } else if (e.key === 'Escape') {
      searchInput.value = '';
      actionSubmitBtn.classList.remove('is-active');
      actionSubmitBtn.disabled = true;
      hideModeSelection();
      searchInput.blur();
      setState('ambient');
    }
  });

  // Action Button Click
  actionSubmitBtn.addEventListener('click', () => {
    if (searchInput.value.trim().length > 0) {
      modeCardGeneral.classList.add('is-selected');
      executeGeneralSearch(searchInput.value.trim());
    }
  });

  // =========================================================================
  // Dark / Light Theme Toggle System
  // =========================================================================
  const themeToggleBtn = document.getElementById('themeToggleBtn');
  const themeToggleText = document.getElementById('themeToggleText');

  function applyTheme(theme) {
    if (theme === 'dark') {
      body.classList.add('theme-dark');
      if (themeToggleText) themeToggleText.textContent = 'โหมดสว่าง';
      localStorage.setItem('antigravity_theme', 'dark');
    } else {
      body.classList.remove('theme-dark');
      if (themeToggleText) themeToggleText.textContent = 'โหมดมืด';
      localStorage.setItem('antigravity_theme', 'light');
    }
    // Re-render D3 graph if active so colors match instantly
    if (window.treeRoot && window.updateTree) {
      window.updateTree(window.treeRoot);
    }
  }

  const savedTheme = localStorage.getItem('antigravity_theme') || 'light';
  applyTheme(savedTheme);

  if (themeToggleBtn) {
    themeToggleBtn.addEventListener('click', () => {
      const isDark = body.classList.contains('theme-dark');
      applyTheme(isDark ? 'light' : 'dark');
    });
  }

  // Close Response Drawer
  closeResponseBtn.addEventListener('click', () => {
    responseDrawer.classList.remove('show');
    modeCardGeneral.classList.remove('is-selected');
    modeCardEtymology.classList.remove('is-selected');
  });

  // =========================================================================
  // โหมด 1: ค้นหาทั่วไป (General Search - WACHA API)
  // =========================================================================
  async function executeGeneralSearch(query) {
    setState('active');
    responseStatusText.textContent = `ค้นหาทั่วไป: "${query}"`;
    responseDrawer.classList.add('show');
    responseBody.innerHTML = `<p style="color:var(--text-secondary);">กำลังค้นหาข้อมูลจาก WACHA Engine...</p>`;

    try {
      const res = await fetch('http://127.0.0.1:8080/api/lookup?q=' + encodeURIComponent(query));
      if (!res.ok) throw new Error('Network response was not ok');
      const data = await res.json();
      
      let html = '';
      
      // การตัดคำ
      html += `<div class="response-query-tag" style="background: rgba(35, 101, 150, 0.1); color: var(--accent-blue);">โหมด: ค้นหาทั่วไป (WACHA)</div>`;
      
      if (data.segmentation && data.segmentation.length > 0) {
        html += `<div style="margin-bottom: 12px; display: flex; gap: 8px; flex-wrap: wrap;">`;
        data.segmentation.forEach(t => {
          const color = t.in_vocab ? "var(--accent-blue)" : "#B91C1C";
          const bg = t.in_vocab ? "#F0F5FA" : "#FEF2F2";
          const border = t.in_vocab ? "#CBDDEB" : "#FECACA";
          html += `<span style="padding: 4px 12px; border-radius: 999px; background: ${bg}; border: 1px solid ${border}; color: ${color}; font-size: 0.95rem; font-weight: 500;">${escapeHtml(t.text)}</span>`;
        });
        html += `</div>`;
      }

      // นิยาม
      if (data.entry) {
        html += `<div class="response-highlight-box">
          <p style="font-size: 1.25rem; font-weight: 700; color: var(--text-primary); margin-bottom: 0.4rem;">
            ${escapeHtml(data.entry.word)} <span style="font-size: 0.95rem; color: var(--accent-blue); font-weight: 400;">${escapeHtml(data.entry.pos)}</span>
          </p>
          <p style="font-size: 1.05rem; line-height: 1.5; color: var(--text-secondary);">${escapeHtml(data.entry.definition)}</p>
        </div>`;
      } else {
        html += `<div class="response-highlight-box"><p>ไม่พบนิยามของคำนี้ในพจนานุกรม</p></div>`;
      }

      // คำอธิบายง่าย (Learner)
      if (data.learner) {
        html += `<div class="learner">
          <div class="body">${escapeHtml(data.learner.simple)}</div>
          <div class="example">ตัวอย่าง: ${escapeHtml(data.learner.example)}</div>
          <div class="prov">ที่มา: ${escapeHtml(data.learner.source)}</div>
        </div>`;
      }

      // แผนภาพความสัมพันธ์ (SVG Graph)
      if (data.related && data.related.length > 0) {
        html += `<h4 style="margin-top: 24px; color: var(--text-secondary); text-align: center;">แผนภาพความสัมพันธ์ — คลิกคำเพื่อสำรวจต่อ (explainable graph)</h4>`;
        html += buildGraphSvg(data);
      }

      // คำที่เกี่ยวข้อง (Related)
      if (data.related && data.related.length > 0) {
        html += `<h4 style="margin-top: 20px; color: var(--text-secondary);">คำที่เกี่ยวข้องพร้อมคำอธิบาย (Explainable)</h4>`;
        html += `<ul style="list-style: none; padding: 0; margin-top: 10px;">`;
        data.related.forEach(r => {
          html += `<li style="padding: 14px 0; border-bottom: 1px dashed var(--glass-border);">
            <strong style="color: var(--accent-blue); font-size: 1.35rem; cursor: pointer; font-weight: 700;" class="rw" data-word="${escapeHtml(r.word)}">${escapeHtml(r.word)}</strong>
            <span style="font-size: 1.05rem; color: var(--text-muted); margin-left: 10px; font-weight: 500;">ความสัมพันธ์: ${r.score.toFixed(2)}</span>
            <div style="font-size: 1.16rem; line-height: 1.75; color: var(--text-primary); margin-top: 8px; padding-left: 14px; border-left: 3.5px solid var(--accent-blue);">`;
          r.path.forEach(p => {
             html += `<div style="padding: 2px 0;">↳ ${escapeHtml(p)}</div>`;
          });
          html += `</div></li>`;
        });
        html += `</ul>`;
      }

      responseBody.innerHTML = html;
      
      // Add event listeners to SVG nodes and list items to allow re-searching
      responseBody.querySelectorAll('.rw').forEach(el => {
        el.addEventListener('click', () => {
          const w = el.getAttribute('data-word');
          if (w) {
             searchInput.value = w;
             executeGeneralSearch(w);
             window.scrollTo({ top: 0, behavior: "smooth" });
          }
        });
      });
      
    } catch (e) {
      responseBody.innerHTML = `<p style="color: var(--accent-magenta);">เกิดข้อผิดพลาดในการดึงข้อมูลจาก WACHA: ${escapeHtml(e.message)}</p>`;
    }
  }

  // =========================================================================
  // โหมด 2: การหารากศัพท์ (Etymological Bridge API)
  // =========================================================================
  async function executeEtymologySearch(query) {
    setState('active');
    responseStatusText.textContent = `สืบสายรากศัพท์: "${query}"`;
    responseDrawer.classList.add('show');
    responseBody.innerHTML = `<p style="color:var(--text-secondary);">กำลังค้นหาข้อมูลรากศัพท์จาก Etymological Bridge...</p>`;

    try {
      const res = await fetch('http://127.0.0.1:8089/api/etymology/' + encodeURIComponent(query));
      if (!res.ok) throw new Error('Network response was not ok');
      const data = await res.json();
      
      let html = `<div class="response-query-tag" style="background: rgba(35, 101, 150, 0.1); color: var(--accent-blue);">โหมด: การหารากศัพท์ (Etymological Bridge)</div>`;
      
      if (data.found && data.entry) {
        const entry = data.entry;
        html += `<div class="response-highlight-box" style="border-left-color: var(--accent-blue);">
          <p style="font-size: 1.15rem; font-weight: 700; color: var(--text-primary); margin-bottom: 0.3rem;">
            ภาษาไทย: <span style="color: var(--accent-blue);">${escapeHtml(entry.thai_word)}</span>
          </p>`;
        
        if (entry.sanskrit_word || entry.pali_word || entry.pali_sanskrit_form) {
          html += `<p><strong>ภาษาบาลี-สันสกฤต:</strong> <em>${escapeHtml(entry.pali_sanskrit_form || entry.sanskrit_word || entry.pali_word)}</em></p>`;
        }
        
        if (entry.pie_root) {
          html += `<p><strong>Proto-Indo-European (PIE) Root:</strong> <code>${escapeHtml(entry.pie_root)}</code> ${entry.pie_meaning ? `(ความหมายดั้งเดิม: ${escapeHtml(entry.pie_meaning)})` : ''}</p>`;
        }
        
        // Handle cognates if they are objects
        if (entry.english_cognates && entry.english_cognates.length > 0) {
          const cogList = entry.english_cognates.map(c => typeof c === 'string' ? escapeHtml(c) : escapeHtml(c.word)).join(', ');
          html += `<p><strong>English Cognates (คำร่วมเชื้อสาย):</strong> <span style="color: var(--accent-blue); font-weight: 600;">${cogList}</span></p>`;
        }
        
        html += `</div>`;
        
        if (entry.explanation) {
          html += `<p style="margin-top: 1.1rem; font-size: 1.15rem; line-height: 1.75; color: var(--text-primary);"><strong>คำอธิบายทางภาษาศาสตร์:</strong> ${escapeHtml(entry.explanation)}</p>`;
        } else if (data.classification && data.classification.entry && data.classification.entry.sound_change_law) {
           html += `<p style="margin-top: 1.1rem; font-size: 1.15rem; line-height: 1.75; color: var(--text-primary);"><strong>การวิเคราะห์สัทศาสตร์เชิงประวัติศาสตร์:</strong> ${escapeHtml(data.classification.entry.sound_change_law)}</p>`;
        }

        // 1. แผนภาพรากศัพท์ (Etymology Tree) — ย้ายขึ้นมาก่อน Timeline ตามที่ผู้ใช้สั่ง
        html += `<h4 style="margin-top: 28px; color: var(--text-secondary); text-align: center; font-size: 1.25rem;">แผนภาพรากศัพท์ (Etymology Tree)</h4>`;
        html += `<div class="d3-graph-wrapper">
                   <div class="d3-graph-toolbar">
                     <span class="d3-graph-tip">คลิกโหนดเพื่อขยาย/ย่อ หรือลากเพื่อเลื่อนมุมมอง</span>
                     <div class="d3-graph-actions">
                       <button type="button" class="d3-btn" id="graphResetZoomBtn">รีเซ็ตมุมมอง</button>
                       <button type="button" class="d3-btn" id="graphExpandAllBtn">ขยายทั้งหมด</button>
                       <button type="button" class="d3-btn" id="graphCollapseAllBtn">ย่อทั้งหมด</button>
                     </div>
                   </div>
                   <div id="treeBreadcrumb" class="tree-breadcrumb"></div>
                   <div style="width: 100%; height: 500px; position: relative;">
                     <svg id="d3GraphSvg" width="100%" height="100%" preserveAspectRatio="xMidYMid meet"></svg>
                   </div>
                   <div id="selectedNodeCard" class="selected-node-card" style="display: none;">
                     <div class="node-card-header">
                       <h5 id="nodeDetailTitle" class="node-card-title"></h5>
                       <span id="nodeDetailBadge" class="node-card-badge"></span>
                       <span id="nodeDetailEra" class="node-card-era"></span>
                     </div>
                     <div id="nodeDetailBody" class="node-card-body"></div>
                     <div id="nodeDetailTip" class="node-card-tip"></div>
                   </div>
                 </div>`;

        // 2. วิวัฒนาการคำ (Timeline Tracing)
        if (entry.timeline && entry.timeline.length > 0) {
           html += `<h4 style="margin-top: 30px; color: var(--text-secondary); font-size: 1.25rem;">วิวัฒนาการคำ (Timeline)</h4>`;
           html += `<div style="margin-top: 12px; padding-left: 18px; border-left: 2.5px solid var(--accent-blue);">`;
           entry.timeline.forEach(t => {
              html += `<div style="margin-bottom: 14px; position: relative;">`;
              html += `<div style="position: absolute; left: -24px; top: 5px; width: 11px; height: 11px; border-radius: 50%; background: var(--accent-blue);"></div>`;
              html += `<strong style="color: var(--text-primary); font-size: 1.15rem;">${escapeHtml(t.stage || '')} (${escapeHtml(t.era || '')})</strong><br>`;
              html += `<span style="color: var(--accent-blue); font-family: monospace; font-size: 1.25rem; font-weight: 700;">${escapeHtml(t.form || '')}</span>`;
              if (t.meaning) html += ` <span style="color: var(--text-secondary); font-size: 1.1rem;">— ${escapeHtml(t.meaning)}</span>`;
              html += `</div>`;
           });
           html += `</div>`;
        }

        // 3. เส้นทางคำร่วมเชื้อสาย (Cognate Derivation Paths)
        if (entry.english_cognates && entry.english_cognates.length > 0 && typeof entry.english_cognates[0] === 'object') {
           html += `<h4 style="margin-top: 30px; color: var(--text-secondary); font-size: 1.25rem;">เส้นทางคำร่วมเชื้อสาย (Derivation Paths)</h4>`;
           html += `<ul style="list-style: none; padding: 0; margin-top: 12px;">`;
           entry.english_cognates.forEach(c => {
             html += `<li class="cognate-card-item">`;
             html += `<strong class="cognate-word">${escapeHtml(c.word)}</strong> <span class="cognate-lang">(${escapeHtml(c.origin_language || '')})</span><br>`;
             html += `<div class="derivation-path"><strong>เส้นทาง:</strong> ${escapeHtml(c.derivation_path || '')}</div>`;
             if (c.usage_note) html += `<div class="note-badge"><strong>Note:</strong> ${escapeHtml(c.usage_note)}</div>`;
             html += `</li>`;
           });
           html += `</ul>`;
        }

      } else {
        html += `<div class="response-highlight-box" style="border-left-color: var(--accent-blue);">
          <p>ไม่พบข้อมูลรากศัพท์สำหรับ "${escapeHtml(query)}" ในฐานข้อมูล Etymological Bridge</p>
          <p style="font-size: 0.9rem; color: #64748B;">คำนี้อาจเป็นคำไทยแท้ (Kra-Dai) หรือไม่อยู่ในคลังคำสาธิต</p>
        </div>`;
      }
      
      responseBody.innerHTML = html;

      // Fetch and Render D3 Graph if entry found
      if (data.found && data.entry) {
         try {
           const graphRes = await fetch(`http://127.0.0.1:8089/api/graph/${encodeURIComponent(query)}`);
           if (graphRes.ok) {
             const graphData = await graphRes.json();
             if (window.initD3Graph && window.renderD3Graph) {
                window.activeEntry = data.entry;
                window.initD3Graph();
                window.renderD3Graph(graphData);
             }
           }
         } catch(e) {
           console.error("D3 Graph fetch error:", e);
         }
      }

    } catch (e) {
      responseBody.innerHTML = `<p style="color: var(--accent-magenta);">เกิดข้อผิดพลาดในการดึงข้อมูลจาก Etymological Bridge: ${escapeHtml(e.message)}</p>`;
    }
  }

  function escapeHtml(str) {
    return String(str).replace(/[&<>'"]/g, 
      tag => ({
        '&': '&amp;',
        '<': '&lt;',
        '>': '&gt;',
        "'": '&#39;',
        '"': '&quot;'
      }[tag] || tag)
    );
  }

  // =========================================================================
  // WACHA SVG Graph Builder logic
  // =========================================================================
  function relationLabel(rw, queryWord) {
    if (!rw.path || !rw.path.length) return "เกี่ยวข้อง";
    for (const edge of rw.path) {
      const m = edge.match(/^(.*?)\s+--(.+?)-->\s+(.*)$/);
      if (!m) continue;
      const [, a, rel, b] = m.map(s => s.trim());
      if ((a === queryWord && b === rw.word) || (a === rw.word && b === queryWord)) {
        return rel;
      }
    }
    const m0 = rw.path[0].match(/^(.*?)\s+--(.+?)-->\s+(.*)$/);
    return m0 ? m0[2].trim() : "เกี่ยวข้อง";
  }

  function buildGraphSvg(d) {
    const W = 640, H = 440, cx = W / 2, cy = H / 2;
    const rels = d.related.slice(0, 8);
    const scores = rels.map(r => r.score);
    const maxS = Math.max(...scores), minS = Math.min(...scores);
    const span = (maxS - minS) || 1;
    const rMin = 96, rMax = 190;
    const query = d.entry ? d.entry.word : (d.segmentation[0] ? d.segmentation[0].text : "");

    let edges = "", nodes = "";
    rels.forEach((rw, i) => {
      const angle = (2 * Math.PI * i) / rels.length - Math.PI / 2;
      const norm = (rw.score - minS) / span;      
      const radius = rMax - norm * (rMax - rMin);  
      const x = cx + radius * Math.cos(angle);
      const y = cy + radius * Math.sin(angle);
      const nodeR = 20 + norm * 12;                
      const rel = relationLabel(rw, query);
      const mx = cx + (x - cx) * 0.55, my = cy + (y - cy) * 0.55;
      
      edges += `<line x1="${cx}" y1="${cy}" x2="${x}" y2="${y}" class="g-edge" />`;
      edges += `<text x="${mx}" y="${my}" class="g-edge-label">${escapeHtml(rel)}</text>`;

      nodes += `<g class="g-node rw" data-word="${escapeHtml(rw.word)}" tabindex="0" role="button">`
        + `<circle cx="${x}" cy="${y}" r="${nodeR}"/>`
        + `<text x="${x}" y="${y + 4}" class="g-node-label">${escapeHtml(rw.word)}</text>`
        + `</g>`;
    });

    const center = `<g class="g-center"><circle cx="${cx}" cy="${cy}" r="34"/>`
      + `<text x="${cx}" y="${cy + 5}" class="g-center-label">${escapeHtml(query)}</text></g>`;

    return `<svg class="graph" viewBox="0 0 ${W} ${H}" width="100%" preserveAspectRatio="xMidYMid meet">`
      + `<g class="g-edges">${edges}</g>${center}<g class="g-nodes">${nodes}</g></svg>`
      + `<div class="g-hint">คลิก (หรือกด Enter ที่) คำใด ๆ เพื่อสำรวจความสัมพันธ์ของคำนั้นต่อ · ยิ่งใกล้กลาง = ยิ่งเกี่ยวข้อง</div>`;
  }

  // ==========================================================================
  // Scroll Driven Kinetic Animation:
  // - When scrolling down to read response details:
  //   Headline pushes up, scales up (+20%), and exits past top screen boundary
  //   Response drawer and stage container expand to wide spacious canvas (1200px)
  // - When scrolling back up to search new word:
  //   Headline smoothly returns and shrinks back to original size
  // ==========================================================================
  function updateScrollProgress() {
    const scrollY = window.scrollY || document.documentElement.scrollTop || 0;
    const progress = Math.min(Math.max(scrollY / 180, 0), 1);
    document.documentElement.style.setProperty('--scroll-progress', progress.toFixed(3));
  }

  window.addEventListener('scroll', updateScrollProgress, { passive: true });

  // Handle window resize dynamically
  window.addEventListener('resize', () => {
    updateScrollProgress();
    if (isAssembled && !greetingHeadline.classList.contains('is-docked')) {
      targetParticles.forEach((p, idx) => {
        const targetSlot = document.getElementById(`targetSlot${idx}`);
        if (targetSlot && p) {
          const slotRect = targetSlot.getBoundingClientRect();
          p.el.style.transform = `translate3d(${slotRect.left}px, ${slotRect.top}px, 0)`;
        }
      });
    }
  });

  // Run initialization
  initWordsCloud();
  animateParticles();
  setState('ambient');
});

