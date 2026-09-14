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
  // Configurable backend API base URL
  const API_BASE = window.API_BASE || 'http://100.76.70.14:8090';

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
  const responseStatusText = document.getElementById('responseStatusText');

  const ALL_MODES = [
    { id: 'general', label: 'ค้นหาทั่วไป', desc: 'นิยามและความสัมพันธ์' },
    { id: 'roots', label: 'สืบสายรากศัพท์', desc: 'PIE และไทม์ไลน์ ๓ ยุค' },
    { id: 'naming', label: 'ตั้งชื่อมงคล', desc: 'มงคลและความหมาย' },
    { id: 'writing', label: 'ช่วยเขียน/กวี', desc: 'คำคล้องจองและระดับภาษา' },
    { id: 'translit', label: 'คำทับศัพท์', desc: 'ทับศัพท์ทางการราชบัณฑิต' },
    { id: 'specialized', label: 'ศัพท์เฉพาะทาง', desc: 'ศัพท์บัญญัติวิชาชีพ' }
  ];

  const modeCards = {
    general: document.getElementById('modeCardGeneral'),
    roots: document.getElementById('modeCardEtymology'),
    etymology: document.getElementById('modeCardEtymology'),
    naming: document.getElementById('modeCardNaming'),
    writing: document.getElementById('modeCardWriting'),
    translit: document.getElementById('modeCardTranslit'),
    specialized: document.getElementById('modeCardSpecialized')
  };

  function selectModeCard(modeKey) {
    Object.entries(modeCards).forEach(([k, card]) => {
      if (!card) return;
      if (k === modeKey || (modeKey === 'roots' && k === 'etymology') || (modeKey === 'etymology' && k === 'roots')) {
        card.classList.add('is-selected');
      } else {
        card.classList.remove('is-selected');
      }
    });
  }

  function deselectAllModeCards() {
    Object.values(modeCards).forEach(card => {
      if (card) card.classList.remove('is-selected');
    });
  }

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

  // ขณะพิมพ์: โหลดโหมดให้เลือกใต้ช่องพิมพ์
  searchInput.addEventListener('input', (e) => {
    const value = e.target.value.trim();
    if (value.length > 0) {
      setState('active');
      actionSubmitBtn.classList.add('is-active');
      actionSubmitBtn.disabled = false;

      // โหลดตัวเลือกใต้ช่องพิมพ์ พร้อมใส่คำที่กำลังพิมพ์ลงไปในพรีวิว
      showModeSelection(value);
    } else {
      setState('aligned');
      actionSubmitBtn.classList.remove('is-active');
      actionSubmitBtn.disabled = true;

      // ซ่อนโหมด และซ่อนคำตอบเมื่อลบคำออก
      hideModeSelection();
    }
  });

  // แสดงตัวเลือกโหมดใต้ช่องพิมพ์
  function showModeSelection(word) {
    typedWordLabel.textContent = word;
    echoWordSpans.forEach(span => {
      span.textContent = word;
    });

    modeSelectionContainer.style.display = 'block';
    if (suggestionTray) suggestionTray.style.display = 'none';
  }

  // ซ่อนตัวเลือกโหมด
  function hideModeSelection() {
    modeSelectionContainer.style.display = 'none';
    if (suggestionTray) suggestionTray.style.display = 'flex';
    responseDrawer.classList.remove('show');
    deselectAllModeCards();
  }

  // ผูก Event Listener เมื่อคลิกการ์ดโหมดแต่ละใบ
  ['general', 'roots', 'naming', 'writing', 'translit', 'specialized'].forEach(modeKey => {
    const card = modeCards[modeKey];
    if (!card) return;
    card.addEventListener('click', () => {
      const query = searchInput.value.trim();
      if (!query) return;
      selectModeCard(modeKey);
      executeMode(modeKey, query, null);
    });
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
      routeIntentAndExecute(searchInput.value.trim());
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
      routeIntentAndExecute(searchInput.value.trim());
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
    deselectAllModeCards();
  });

  // =========================================================================
  // Intent Router & Confirmation UI System (Phase 3)
  // =========================================================================
  function renderIntentConfirmation(guess, query) {
    if (!guess) return '';
    const currentIntent = guess.intent || 'general';
    const currentLabel = guess.label || (ALL_MODES.find(m => m.id === currentIntent)?.label || 'ค้นหาทั่วไป');
    const others = ALL_MODES.filter(m => m.id !== currentIntent && m.id !== 'general');
    
    const altButtons = others.map(m => `
      <button type="button" class="intent-alt-chip" data-mode="${m.id}" data-query="${escapeHtml(query)}" style="
        background: var(--glass-bg);
        border: 1px solid var(--glass-border);
        border-radius: 999px;
        padding: 3px 12px;
        font-size: 0.85rem;
        color: var(--accent-blue);
        cursor: pointer;
        margin: 2px 4px;
        transition: all 0.2s;
        font-weight: 500;
      ">
        ${escapeHtml(m.label.split('/')[0])}
      </button>
    `).join('');

    return `
      <div class="intent-confirm-banner" style="
        margin-bottom: 20px;
        padding: 12px 18px;
        border-radius: 12px;
        background: rgba(35, 101, 150, 0.08);
        border: 1.5px solid rgba(35, 101, 150, 0.22);
        box-shadow: 0 2px 10px rgba(15, 41, 66, 0.04);
      ">
        <div style="font-size: 1.02rem; color: var(--text-primary); font-weight: 500;">
          เราคิดว่าคุณอยาก <strong>${escapeHtml(currentLabel)}</strong> — ใช่ไหม?
          <span style="font-size: 0.88rem; color: var(--text-muted); font-weight: 400; margin-left: 6px;">
            (${escapeHtml(guess.reason || '')}${guess.confidence === 'vector' ? ' · เดาจากความหมาย' : ''})
          </span>
        </div>
        <div style="margin-top: 8px; font-size: 0.88rem; color: var(--text-secondary); display: flex; align-items: center; flex-wrap: wrap; gap: 4px;">
          <span>ไม่ใช่? ลอง:</span>
          ${altButtons}
          <button type="button" class="intent-alt-chip" data-mode="general" data-query="${escapeHtml(query)}" style="
            background: var(--glass-bg);
            border: 1px solid var(--glass-border);
            border-radius: 999px;
            padding: 3px 12px;
            font-size: 0.85rem;
            color: var(--text-muted);
            cursor: pointer;
            margin: 2px 4px;
            font-weight: 500;
          ">
            ค้นหาทั่วไป
          </button>
        </div>
      </div>
    `;
  }

  function attachIntentChipListeners() {
    responseBody.querySelectorAll('.intent-alt-chip').forEach(btn => {
      btn.addEventListener('click', () => {
        const targetMode = btn.getAttribute('data-mode');
        const targetQuery = btn.getAttribute('data-query');
        if (targetMode && targetQuery) {
          const altGuess = {
            query: targetQuery,
            intent: targetMode,
            label: ALL_MODES.find(m => m.id === targetMode)?.label || targetMode,
            reason: 'คุณเลือกเอง',
            confidence: 'rule'
          };
          executeMode(targetMode, targetQuery, altGuess);
        }
      });
    });
  }

  async function routeIntentAndExecute(query) {
    query = (query || "").trim();
    if (!query) return;

    setState('active');
    responseStatusText.textContent = `วิเคราะห์เจตนา: "${query}"`;
    responseDrawer.classList.add('show');
    responseBody.innerHTML = `<p style="color:var(--text-secondary);">กำลังวิเคราะห์เจตนาและค้นหาข้อมูลจาก WACHA Engine...</p>`;

    let guess = null;
    try {
      const res = await fetch(`${API_BASE}/api/intent?q=` + encodeURIComponent(query));
      if (res.ok) {
        guess = await res.json();
      }
    } catch (err) {
      console.warn("Intent router network error, falling back to general search:", err);
    }

    if (!guess || !guess.intent) {
      guess = { query, intent: 'general', label: 'ค้นหาทั่วไป', reason: 'ค้นหาทั่วไป', confidence: 'default' };
    }

    const mode = guess.intent;
    await executeMode(mode, query, guess);
  }

  async function executeMode(modeKey, query, guess) {
    selectModeCard(modeKey);
    switch (modeKey) {
      case 'naming':
        await executeNamingSearch(query, guess);
        break;
      case 'roots':
      case 'etymology':
        await executeEtymologySearch(query, guess);
        break;
      case 'writing':
        await executeWritingSearch(query, guess);
        break;
      case 'translit':
        await executeTranslitSearch(query, guess);
        break;
      case 'specialized':
        await executeSpecializedSearch(query, guess);
        break;
      case 'general':
      default:
        await executeGeneralSearch(query, guess);
        break;
    }
  }

  // =========================================================================
  // โหมด 1: ค้นหาทั่วไป (General Search - WACHA API)
  // =========================================================================
  async function executeGeneralSearch(query, guess = null) {
    setState('active');
    responseStatusText.textContent = `ค้นหาทั่วไป: "${query}"`;
    responseDrawer.classList.add('show');
    responseBody.innerHTML = `<p style="color:var(--text-secondary);">กำลังค้นหาข้อมูลจาก WACHA Engine...</p>`;

    try {
      const res = await fetch(`${API_BASE}/api/lookup?q=` + encodeURIComponent(query));
      if (!res.ok) throw new Error('Network response was not ok');
      const data = await res.json();
      
      let html = '';
      if (guess) html += renderIntentConfirmation(guess, query);
      
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
            ${escapeHtml(data.entry.word)} 
            <span style="font-size: 0.95rem; color: var(--accent-blue); font-weight: 400;">${escapeHtml(data.entry.pos || '')}</span>
            ${data.entry.register ? `<span style="font-size: 0.85rem; color: var(--text-muted); font-weight: 400; margin-left: 4px;">(${escapeHtml(data.entry.register)})</span>` : ''}
            ${data.entry.source ? `<span style="font-size: 0.8rem; color: var(--text-muted); opacity: 0.75; margin-left: 8px;">[${escapeHtml(data.entry.source)}]</span>` : ''}
          </p>
          <p style="font-size: 1.05rem; line-height: 1.5; color: var(--text-secondary);">${escapeHtml(data.entry.definition || '')}</p>`;
        
        if (data.entry.examples && data.entry.examples.length > 0) {
          html += `<div style="margin-top: 8px; font-size: 0.95rem; color: var(--text-secondary);"><em>ตัวอย่าง: ${data.entry.examples.map(ex => escapeHtml(ex)).join(', ')}</em></div>`;
        }
        html += `</div>`;
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
            <span style="font-size: 1.05rem; color: var(--text-muted); margin-left: 10px; font-weight: 500;">ความสัมพันธ์: ${typeof r.score === 'number' ? r.score.toFixed(2) : r.score}</span>
            <div style="font-size: 1.16rem; line-height: 1.75; color: var(--text-primary); margin-top: 8px; padding-left: 14px; border-left: 3.5px solid var(--accent-blue);">`;
          (r.path || []).forEach(p => {
             html += `<div style="padding: 2px 0;">↳ ${escapeHtml(p)}</div>`;
          });
          html += `</div></li>`;
        });
        html += `</ul>`;
      }

      responseBody.innerHTML = html;
      attachIntentChipListeners();
      
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
  // โหมด 2: การหารากศัพท์ (Etymology / Roots Mode - WACHA API)
  // =========================================================================
  async function executeEtymologySearch(query, guess = null) {
    setState('active');
    responseStatusText.textContent = `สืบสายรากศัพท์: "${query}"`;
    responseDrawer.classList.add('show');
    responseBody.innerHTML = `<p style="color:var(--text-secondary);">กำลังค้นหาข้อมูลรากศัพท์จาก WACHA Engine...</p>`;

    try {
      const res = await fetch(`${API_BASE}/api/lookup?q=` + encodeURIComponent(query));
      if (!res.ok) throw new Error('Network response was not ok');
      const data = await res.json();
      
      let html = '';
      if (guess) html += renderIntentConfirmation(guess, query);
      html += `<div class="response-query-tag" style="background: rgba(35, 101, 150, 0.1); color: var(--accent-blue);">โหมด: การหารากศัพท์และวิวัฒนาการ (Etymology & Roots)</div>`;
      
      if (data.entry) {
        const entry = data.entry;
        const cognates = entry.english_cognates || [];
        const pieRoot = cognates.find(c => c.pie_root)?.pie_root || '';
        
        html += `<div class="response-highlight-box" style="border-left-color: var(--accent-blue);">
          <p style="font-size: 1.25rem; font-weight: 700; color: var(--text-primary); margin-bottom: 0.3rem;">
            ภาษาไทย: <span style="color: var(--accent-blue);">${escapeHtml(entry.word)}</span>
            ${entry.pos ? `<span style="font-size: 0.95rem; color: var(--text-muted); font-weight: 400; margin-left: 6px;">${escapeHtml(entry.pos)}</span>` : ''}
            ${entry.register ? `<span style="font-size: 0.85rem; color: var(--text-muted); font-weight: 400; margin-left: 4px;">(${escapeHtml(entry.register)})</span>` : ''}
          </p>
          <p style="font-size: 1.05rem; line-height: 1.5; color: var(--text-secondary); margin-bottom: 0.6rem;">${escapeHtml(entry.definition || '')}</p>`;
        
        // Etymology loans
        if (entry.etymology && entry.etymology.length > 0) {
          const etymList = entry.etymology.map(et => {
            const langLabel = et.lang ? `[${escapeHtml(et.lang)}] ` : '';
            return `<span style="color: var(--accent-blue); font-weight: 600;">${langLabel}${escapeHtml(et.form)}</span>`;
          }).join('; ');
          html += `<p style="margin-top: 6px;"><strong>รากศัพท์ / ภาษาที่มา:</strong> ${etymList}</p>`;
        }
        
        // PIE root
        if (pieRoot) {
          html += `<p style="margin-top: 6px;"><strong>Proto-Indo-European (PIE) Root:</strong> <code style="font-family: monospace; font-size: 1.05rem; background: rgba(35, 101, 150, 0.08); padding: 2px 6px; border-radius: 4px; color: var(--accent-blue); font-weight: 600;">${escapeHtml(pieRoot)}</code></p>`;
        }
        
        // English cognates
        if (cognates.length > 0) {
          const cogList = cognates.map(c => `<span style="display: inline-block; margin: 2px 4px; padding: 2px 10px; border-radius: 12px; background: rgba(35, 101, 150, 0.1); color: var(--accent-blue); font-weight: 600;">${escapeHtml(c.word)}</span>`).join(' ');
          html += `<p style="margin-top: 6px;"><strong>English Cognates (คำร่วมเชื้อสาย):</strong> ${cogList}</p>`;
        } else {
          html += `<p style="margin-top: 6px; color: var(--text-muted); font-size: 0.95rem;"><em>คำนี้เป็นคำในตระกูลภาษาขร้า-ไท (Kra-Dai) หรือไม่มีรากศัพท์ร่วมกับสายภาษาอินโด-ยูโรเปียน (PIE)</em></p>`;
        }

        // Sub entries
        if (entry.sub_entries && entry.sub_entries.length > 0) {
          html += `<p style="margin-top: 6px; font-size: 0.95rem; color: var(--text-secondary);"><strong>ลูกคำ:</strong> ${entry.sub_entries.map(s => escapeHtml(s)).join(', ')}</p>`;
        }
        
        html += `</div>`;

        // 1. แผนภาพรากศัพท์และเครือข่ายความสัมพันธ์ (Interactive D3 Tree)
        html += `<h4 style="margin-top: 28px; color: var(--text-secondary); text-align: center; font-size: 1.25rem;">แผนภาพรากศัพท์และเครือข่ายความสัมพันธ์ (Interactive Tree)</h4>`;
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

        // 2. วิวัฒนาการคำในพจนานุกรม ๓ ยุค (Timeline Tracing)
        let timeline = (data.evolution && data.evolution.length) ? data.evolution : [];
        if (!timeline.length) {
          try {
            const evoRes = await fetch(`${API_BASE}/api/evolution?q=` + encodeURIComponent(query));
            if (evoRes.ok) {
              const evoData = await evoRes.json();
              timeline = evoData.timeline || [];
            }
          } catch (_) {}
        }

        if (timeline && timeline.length > 0) {
          html += `<h4 style="margin-top: 30px; color: var(--text-secondary); font-size: 1.25rem;">วิวัฒนาการคำในพจนานุกรม ๓ ยุค (๒๕๔๒ → ๒๕๕๔ → ๒๕๖๙)</h4>`;
          html += `<div style="margin-top: 12px; padding-left: 18px; border-left: 3px solid var(--accent-blue);">`;
          timeline.forEach(t => {
            html += `<div style="margin-bottom: 14px; position: relative;">`;
            html += `<div style="position: absolute; left: -24px; top: 5px; width: 11px; height: 11px; border-radius: 50%; background: var(--accent-blue);"></div>`;
            html += `<strong style="color: var(--text-primary); font-size: 1.15rem;">พจนานุกรม พ.ศ. ${escapeHtml(t.edition)}</strong><br>`;
            html += `<div style="color: var(--text-secondary); margin-top: 4px; line-height: 1.6; font-size: 1.05rem;">${escapeHtml(t.definition)}</div>`;
            if (t.is_draft) {
              html += `<div style="margin-top: 6px; display: inline-block; padding: 2px 8px; border-radius: 4px; background: rgba(217, 119, 6, 0.12); border: 1px solid #F59E0B; color: #D97706; font-size: 0.9rem; font-weight: 600;">⚠ ${escapeHtml(t.draft_label || 'ร่าง อยู่ระหว่างดำเนินการ ยังไม่เป็นข้อมูลทางการ')}</div>`;
            }
            html += `</div>`;
          });
          html += `</div>`;
          html += `<div style="margin-top: 8px; font-size: 0.85rem; color: var(--text-muted);">ราชบัณฑิตยสภาชำระพจนานุกรมต่อเนื่อง ๒๕๔๒ → ๒๕๕๔ → ๒๕๖๙</div>`;
        }

        // 3. เส้นทางคำร่วมเชื้อสาย (Derivation Paths)
        if (cognates.length > 0) {
          html += `<h4 style="margin-top: 30px; color: var(--text-secondary); font-size: 1.25rem;">เส้นทางคำร่วมเชื้อสาย (Derivation Paths)</h4>`;
          html += `<ul style="list-style: none; padding: 0; margin-top: 12px;">`;
          cognates.forEach(c => {
            html += `<li class="cognate-card-item">`;
            html += `<strong class="cognate-word">${escapeHtml(c.word)}</strong> <span class="cognate-lang">(PIE: ${escapeHtml(c.pie_root || pieRoot || '')})</span><br>`;
            html += `<div class="derivation-path"><strong>เส้นทาง:</strong> ${escapeHtml(c.pie_root ? `${c.pie_root} ➔ ${c.word}` : c.word)}</div>`;
            html += `</li>`;
          });
          html += `</ul>`;
        }

      } else {
        html += `<div class="response-highlight-box" style="border-left-color: var(--accent-blue);">
          <p>ไม่พบข้อมูลรากศัพท์สำหรับ "${escapeHtml(query)}" ในคลังข้อมูล WACHA</p>
          <p style="font-size: 0.9rem; color: #64748B;">คำนี้อาจเป็นคำเฉพาะ หรือยังไม่อยู่ในคลังคำสาธิต</p>
        </div>`;
      }
      
      responseBody.innerHTML = html;
      attachIntentChipListeners();

      // Render D3 Graph if entry found
      if (data.entry) {
        try {
          if (window.initD3Graph && window.renderD3Graph) {
            const cognates = data.entry.english_cognates || [];
            const pieRoot = cognates.find(c => c.pie_root)?.pie_root || '';
            window.activeEntry = {
              thai_word: data.entry.word,
              word: data.entry.word,
              orst_pos: data.entry.pos,
              pos: data.entry.pos,
              orst_definition: data.entry.definition,
              definition: data.entry.definition,
              pali_sanskrit_form: (data.entry.etymology && data.entry.etymology[0]?.form) || "",
              pali_sanskrit_lang: (data.entry.etymology && data.entry.etymology[0]?.lang) || "บาลี/สันสกฤต",
              pie_root: pieRoot || (data.entry.etymology?.length ? "*PIE" : ""),
              english_cognates: cognates.map(c => ({
                word: c.word,
                origin_language: "Indo-European / English",
                derivation_path: c.pie_root ? `${c.pie_root} ➔ ${c.word}` : c.word,
                usage_note: "คำร่วมสายตระกูลภาษาอินโด-ยูโรเปียน",
                difficulty: "General"
              })),
              related: data.related || []
            };
            window.initD3Graph();
            window.renderD3Graph({ thai_word: data.entry.word, pie_root: window.activeEntry.pie_root, related: data.related });
          }
        } catch(e) {
          console.error("D3 Graph render error:", e);
        }
      }

    } catch (e) {
      responseBody.innerHTML = `<p style="color: var(--accent-magenta);">เกิดข้อผิดพลาดในการดึงข้อมูลจาก WACHA: ${escapeHtml(e.message)}</p>`;
    }
  }

  // =========================================================================
  // โหมด 3: คลังคำเพื่อตั้งชื่อ (Naming Mode - Reverse + Lookup API)
  // =========================================================================
  async function executeNamingSearch(query, guess = null) {
    setState('active');
    responseStatusText.textContent = `ตั้งชื่อมงคล: "${query}"`;
    responseDrawer.classList.add('show');
    responseBody.innerHTML = `<p style="color:var(--text-secondary);">กำลังค้นหาชื่อและความหมายมงคลจาก WACHA Engine...</p>`;

    try {
      let lookupData = null;
      let reverseData = null;

      const [revRes, lkpRes] = await Promise.all([
        fetch(`${API_BASE}/api/reverse?q=` + encodeURIComponent(query)).catch(() => null),
        fetch(`${API_BASE}/api/lookup?q=` + encodeURIComponent(query)).catch(() => null)
      ]);

      if (revRes && revRes.ok) reverseData = await revRes.json();
      if (lkpRes && lkpRes.ok) lookupData = await lkpRes.json();

      let html = '';
      if (guess) html += renderIntentConfirmation(guess, query);
      html += `<div class="response-query-tag" style="background: rgba(217, 119, 6, 0.1); color: #D97706;">โหมด: คลังคำเพื่อตั้งชื่อ (Naming Engine)</div>`;

      // 1. Direct Entry Profile if available
      if (lookupData && lookupData.entry) {
        const entry = lookupData.entry;
        html += `<div class="response-highlight-box" style="border-left-color: #D97706;">
          <p style="font-size: 1.25rem; font-weight: 700; color: var(--text-primary); margin-bottom: 0.3rem;">
            ${escapeHtml(entry.word)} 
            <span style="font-size: 0.95rem; color: #D97706; font-weight: 500;">${escapeHtml(entry.pos || '')}</span>
            ${entry.register ? `<span style="font-size: 0.85rem; color: var(--text-muted); font-weight: 400; margin-left: 4px;">(${escapeHtml(entry.register)})</span>` : ''}
          </p>
          <p style="font-size: 1.05rem; line-height: 1.5; color: var(--text-secondary); margin-bottom: 0.6rem;">${escapeHtml(entry.definition || '')}</p>`;
        
        if (entry.etymology && entry.etymology.length > 0) {
          const etymList = entry.etymology.map(et => {
            const l = et.lang ? `[${escapeHtml(et.lang)}] ` : '';
            return `<span style="color: #D97706; font-weight: 600;">${l}${escapeHtml(et.form)}</span>`;
          }).join('; ');
          html += `<p style="margin-top: 6px;"><strong>รากศัพท์มงคล (บาลี-สันสกฤต):</strong> ${etymList}</p>`;
        }
        
        if (entry.english_cognates && entry.english_cognates.length > 0) {
          const cogs = entry.english_cognates.map(c => `<span style="color: var(--accent-blue); font-weight: 600;">${escapeHtml(c.word)}</span>`).join(', ');
          html += `<p style="margin-top: 6px;"><strong>คำร่วมสาย PIE:</strong> ${cogs}</p>`;
        }
        html += `</div>`;
      }

      // 2. Reverse Dictionary Naming Candidates (BM25 Hits)
      if (reverseData && reverseData.hits && reverseData.hits.length > 0) {
        html += `<h4 style="margin-top: 24px; color: var(--text-secondary); font-size: 1.2rem;">รายชื่อและคำศัพท์ที่มีความหมายสอดคล้อง (BM25 Reverse Index)</h4>`;
        html += `<ul style="list-style: none; padding: 0; margin-top: 10px;">`;
        reverseData.hits.forEach((h, i) => {
          const matchedBadges = (h.matched || []).map(m => `<span style="display: inline-block; padding: 2px 8px; border-radius: 4px; background: rgba(217, 119, 6, 0.1); color: #D97706; font-size: 0.85rem; margin-right: 4px;">ตรงคำ: ${escapeHtml(m)}</span>`).join('');
          html += `<li style="padding: 12px 0; border-bottom: 1px dashed var(--glass-border); display: flex; align-items: flex-start; justify-content: space-between; gap: 12px;">
            <div>
              <span style="font-size: 1.2rem; font-weight: 700; color: #D97706; cursor: pointer;" class="name-link rw" data-word="${escapeHtml(h.word)}">${i + 1}. ${escapeHtml(h.word)}</span>
              <span style="font-size: 0.9rem; color: var(--text-muted); margin-left: 8px;">คะแนน: ${h.score.toFixed(2)}</span>
              <div style="margin-top: 4px;">${matchedBadges}</div>
            </div>
            <button type="button" class="d3-btn rw" data-word="${escapeHtml(h.word)}" style="white-space: nowrap; font-size: 0.82rem; padding: 4px 10px;">ดูความหมาย →</button>
          </li>`;
        });
        html += `</ul>`;
        html += `<div style="margin-top: 8px; font-size: 0.85rem; color: var(--text-muted);">ดัชนีผกผัน BM25 ค้นหาจากนิยามพจนานุกรมเพื่อถอดความหมายเป็นชื่อมงคล · คลิกคำเพื่อเปิดความหมาย</div>`;
      } else if (!lookupData || !lookupData.entry) {
        html += `<div class="response-highlight-box" style="border-left-color: #D97706;">
          <p>ไม่พบรายชื่อหรือคำที่มีความหมายตรงกับ "${escapeHtml(query)}" ในคลังคำตั้งชื่อ</p>
          <p style="font-size: 0.9rem; color: #64748B;">ลองค้นหาด้วยคำคุณลักษณะ เช่น "ทอง", "แสงสว่าง", "ความสุข", "ปัญญา"</p>
        </div>`;
      }

      // 3. Related words if lookup had them
      if (lookupData && lookupData.related && lookupData.related.length > 0) {
        html += `<h4 style="margin-top: 24px; color: var(--text-secondary);">คำพ้องและความหมายข้างเคียง</h4>`;
        html += `<div style="display: flex; gap: 8px; flex-wrap: wrap; margin-top: 8px;">`;
        lookupData.related.slice(0, 10).forEach(r => {
          html += `<span class="rw" data-word="${escapeHtml(r.word)}" style="padding: 4px 12px; border-radius: 999px; background: #FFFBEB; border: 1px solid #FDE68A; color: #D97706; font-size: 0.95rem; font-weight: 500; cursor: pointer;">${escapeHtml(r.word)}</span>`;
        });
        html += `</div>`;
      }

      responseBody.innerHTML = html;
      attachIntentChipListeners();

      // Click on any word to search that word
      responseBody.querySelectorAll('.rw').forEach(el => {
        el.addEventListener('click', () => {
          const w = el.getAttribute('data-word');
          if (w) {
            searchInput.value = w;
            executeNamingSearch(w, null);
            window.scrollTo({ top: 0, behavior: "smooth" });
          }
        });
      });

    } catch (e) {
      responseBody.innerHTML = `<p style="color: var(--accent-magenta);">เกิดข้อผิดพลาดในการดึงข้อมูลตั้งชื่อ: ${escapeHtml(e.message)}</p>`;
    }
  }

  // =========================================================================
  // โหมด 4: ช่วยเขียนและงานกวี (Writing & Rhymes Mode)
  // =========================================================================
  async function executeWritingSearch(query, guess = null) {
    setState('active');
    responseStatusText.textContent = `ช่วยเขียน/คำคล้องจอง: "${query}"`;
    responseDrawer.classList.add('show');
    responseBody.innerHTML = `<p style="color:var(--text-secondary);">กำลังค้นหาคำคล้องจองและระดับภาษาจาก WACHA Engine...</p>`;

    try {
      const [rhymeRes, regRes] = await Promise.all([
        fetch(`${API_BASE}/api/rhyme?q=` + encodeURIComponent(query)).catch(() => null),
        fetch(`${API_BASE}/api/register?reg=แบบ&q=` + encodeURIComponent(query)).catch(() => null)
      ]);

      const rhymeData = rhymeRes && rhymeRes.ok ? await rhymeRes.json() : null;
      const regData = regRes && regRes.ok ? await regRes.json() : null;

      let html = '';
      if (guess) html += renderIntentConfirmation(guess, query);
      html += `<div class="response-query-tag" style="background: rgba(79, 70, 229, 0.1); color: #4F46E5;">โหมด: ช่วยเขียนและงานกวี (Writing & Rhymes)</div>`;

      // 1. Loose Rhymes Panel
      html += `<div style="margin-bottom: 24px;">
        <h4 style="color: var(--text-secondary); font-size: 1.2rem; display: flex; align-items: center; gap: 8px;">
          <span>คำคล้องจอง (Loose Rhyme): “<strong style="color: #4F46E5;">${escapeHtml(query)}</strong>”</span>
        </h4>`;

      if (rhymeData && rhymeData.rhymes && rhymeData.rhymes.length > 0) {
        html += `<div style="display: flex; gap: 8px; flex-wrap: wrap; margin-top: 10px;">`;
        rhymeData.rhymes.forEach(w => {
          html += `<span class="rw" data-word="${escapeHtml(w)}" style="padding: 6px 14px; border-radius: 999px; background: #EEF2FF; border: 1px solid #C7D2FE; color: #4F46E5; font-size: 1rem; font-weight: 500; cursor: pointer; transition: all 0.2s;">${escapeHtml(w)}</span>`;
        });
        html += `</div>`;
        html += `<div style="margin-top: 8px; font-size: 0.85rem; color: var(--text-muted);">คล้องจองแบบหลวม (สระ + มาตราตัวสะกดเดียวกัน) · คลิกคำเพื่อดูนิยาม</div>`;
      } else {
        html += `<p style="margin-top: 8px; color: var(--text-muted); font-size: 0.95rem;">ไม่พบคำคล้องจองโดยตรงสำหรับคำนี้</p>`;
      }
      html += `</div>`;

      // 2. Register Filter Panel
      html += `<div style="margin-top: 24px; padding-top: 20px; border-top: 1px solid var(--glass-border);">
        <h4 style="color: var(--text-secondary); font-size: 1.2rem;">กรองระดับภาษา (Register Filter)</h4>
        <div style="display: flex; gap: 6px; flex-wrap: wrap; margin-top: 10px;" id="registerBtnGroup">
          <button type="button" class="reg-btn is-active" data-reg="แบบ" style="padding: 4px 14px; border-radius: 8px; border: 1px solid #4F46E5; background: #4F46E5; color: white; cursor: pointer; font-size: 0.9rem;">ภาษาแบบแผน (แบบ)</button>
          <button type="button" class="reg-btn" data-reg="ราชา" style="padding: 4px 14px; border-radius: 8px; border: 1px solid var(--glass-border); background: var(--glass-bg); color: var(--text-primary); cursor: pointer; font-size: 0.9rem;">ราชาศัพท์ (ราชา)</button>
          <button type="button" class="reg-btn" data-reg="โบ" style="padding: 4px 14px; border-radius: 8px; border: 1px solid var(--glass-border); background: var(--glass-bg); color: var(--text-primary); cursor: pointer; font-size: 0.9rem;">คำโบราณ (โบ)</button>
          <button type="button" class="reg-btn" data-reg="ปาก" style="padding: 4px 14px; border-radius: 8px; border: 1px solid var(--glass-border); background: var(--glass-bg); color: var(--text-primary); cursor: pointer; font-size: 0.9rem;">ภาษาปาก (ปาก)</button>
          <button type="button" class="reg-btn" data-reg="เลิก" style="padding: 4px 14px; border-radius: 8px; border: 1px solid var(--glass-border); background: var(--glass-bg); color: var(--text-primary); cursor: pointer; font-size: 0.9rem;">คำเลิกใช้ (เลิก)</button>
        </div>
        <div id="registerWordsList" style="display: flex; gap: 8px; flex-wrap: wrap; margin-top: 14px;">`;

      if (regData && regData.words && regData.words.length > 0) {
        regData.words.forEach(x => {
          html += `<span class="rw" data-word="${escapeHtml(x.word)}" style="padding: 4px 12px; border-radius: 999px; background: #F8FAFC; border: 1px solid #E2E8F0; color: var(--text-primary); font-size: 0.95rem; cursor: pointer;">${escapeHtml(x.word)}</span>`;
        });
      } else {
        html += `<span style="color: var(--text-muted); font-size: 0.9rem;">ไม่พบคำในระดับภาษานี้</span>`;
      }
      html += `</div>
        <div style="margin-top: 8px; font-size: 0.85rem; color: var(--text-muted);">กรองตามทะเบียนคำในพจนานุกรม (RID) · เรียงตามความถี่</div>
      </div>`;

      responseBody.innerHTML = html;
      attachIntentChipListeners();

      // Word click to lookup
      responseBody.querySelectorAll('.rw').forEach(el => {
        el.addEventListener('click', () => {
          const w = el.getAttribute('data-word');
          if (w) {
            searchInput.value = w;
            executeGeneralSearch(w, null);
            window.scrollTo({ top: 0, behavior: "smooth" });
          }
        });
      });

      // Register button click to switch register on the fly
      responseBody.querySelectorAll('.reg-btn').forEach(btn => {
        btn.addEventListener('click', async () => {
          responseBody.querySelectorAll('.reg-btn').forEach(b => {
            b.style.background = 'var(--glass-bg)';
            b.style.color = 'var(--text-primary)';
            b.style.borderColor = 'var(--glass-border)';
          });
          btn.style.background = '#4F46E5';
          btn.style.color = 'white';
          btn.style.borderColor = '#4F46E5';

          const reg = btn.getAttribute('data-reg');
          const listEl = document.getElementById('registerWordsList');
          if (listEl) listEl.innerHTML = `<span style="color: var(--text-muted);">กำลังโหลด...</span>`;
          try {
            const rRes = await fetch(`${API_BASE}/api/register?reg=${encodeURIComponent(reg)}&q=${encodeURIComponent(query)}`);
            if (rRes.ok) {
              const rData = await rRes.json();
              if (rData.words && rData.words.length > 0) {
                listEl.innerHTML = rData.words.map(x => `
                  <span class="rw" data-word="${escapeHtml(x.word)}" style="padding: 4px 12px; border-radius: 999px; background: #F8FAFC; border: 1px solid #E2E8F0; color: var(--text-primary); font-size: 0.95rem; cursor: pointer;">${escapeHtml(x.word)}</span>
                `).join(' ');
                listEl.querySelectorAll('.rw').forEach(el => {
                  el.addEventListener('click', () => {
                    const w = el.getAttribute('data-word');
                    if (w) {
                      searchInput.value = w;
                      executeGeneralSearch(w, null);
                    }
                  });
                });
              } else {
                listEl.innerHTML = `<span style="color: var(--text-muted); font-size: 0.9rem;">ไม่พบคำในระดับภาษานี้</span>`;
              }
            }
          } catch (_) {}
        });
      });

    } catch (e) {
      responseBody.innerHTML = `<p style="color: var(--accent-magenta);">เกิดข้อผิดพลาดในการดึงข้อมูลช่วยเขียน: ${escapeHtml(e.message)}</p>`;
    }
  }

  // =========================================================================
  // โหมด 5: คำทับศัพท์ทางการ (Transliteration Mode - ORST)
  // =========================================================================
  async function executeTranslitSearch(query, guess = null) {
    setState('active');
    responseStatusText.textContent = `คำทับศัพท์ทางการ: "${query}"`;
    responseDrawer.classList.add('show');
    responseBody.innerHTML = `<p style="color:var(--text-secondary);">กำลังค้นหาคำทับศัพท์จากราชบัณฑิตยสภา...</p>`;

    try {
      const res = await fetch(`${API_BASE}/api/translit?q=` + encodeURIComponent(query));
      if (!res.ok) throw new Error('Network response was not ok');
      const data = await res.json();

      let html = '';
      if (guess) html += renderIntentConfirmation(guess, query);
      html += `<div class="response-query-tag" style="background: rgba(5, 150, 105, 0.1); color: #059669;">โหมด: คำทับศัพท์ทางการ (Transliteration - ORST)</div>`;

      if (data.hits && data.hits.length > 0) {
        html += `<h4 style="margin-top: 16px; color: var(--text-secondary); font-size: 1.2rem;">คำทับศัพท์ทางการ: “${escapeHtml(query)}”</h4>`;
        html += `<ul style="list-style: none; padding: 0; margin-top: 12px;">`;
        data.hits.forEach(h => {
          html += `<li style="padding: 14px 16px; margin-bottom: 10px; border-radius: 12px; background: rgba(5, 150, 105, 0.05); border: 1px solid rgba(5, 150, 105, 0.2);">
            <div style="font-size: 1.3rem; font-weight: 700; color: var(--text-primary); display: flex; align-items: center; gap: 12px; flex-wrap: wrap;">
              <span style="font-family: sans-serif;">${escapeHtml(h.english)}</span>
              <span style="color: #059669; font-size: 1.1rem;">⇄</span>
              <span style="color: #059669;">${escapeHtml(h.thai)}</span>
            </div>
            ${h.note ? `<div style="margin-top: 6px; font-size: 0.95rem; color: var(--text-secondary);">หมายเหตุ: ${escapeHtml(h.note)}</div>` : ''}
          </li>`;
        });
        html += `</ul>`;
        if (data.source) {
          html += `<div style="margin-top: 8px; font-size: 0.85rem; color: var(--text-muted);">${escapeHtml(data.source)}</div>`;
        }
      } else {
        html += `<div class="response-highlight-box" style="border-left-color: #059669;">
          <p>ไม่พบคำทับศัพท์ทางการของ "${escapeHtml(query)}" ในฐานข้อมูลราชบัณฑิตยสภา</p>
          <p style="font-size: 0.9rem; color: #64748B;">รองรับการค้นหาทั้งภาษาไทย (เช่น "คอมพิวเตอร์") และภาษาอังกฤษ (เช่น "internet")</p>
        </div>`;
      }

      responseBody.innerHTML = html;
      attachIntentChipListeners();

    } catch (e) {
      responseBody.innerHTML = `<p style="color: var(--accent-magenta);">เกิดข้อผิดพลาดในการดึงข้อมูลคำทับศัพท์: ${escapeHtml(e.message)}</p>`;
    }
  }

  // =========================================================================
  // โหมด 6: ศัพท์เฉพาะทาง/บัญญัติ (Specialized Terminology Mode)
  // =========================================================================
  async function executeSpecializedSearch(query, guess = null) {
    setState('active');
    responseStatusText.textContent = `ศัพท์เฉพาะทาง/บัญญัติ: "${query}"`;
    responseDrawer.classList.add('show');
    responseBody.innerHTML = `<p style="color:var(--text-secondary);">กำลังค้นหาศัพท์เฉพาะทางจาก WACHA Engine...</p>`;

    try {
      const res = await fetch(`${API_BASE}/api/lookup?q=` + encodeURIComponent(query));
      if (!res.ok) throw new Error('Network response was not ok');
      const data = await res.json();

      let html = '';
      if (guess) html += renderIntentConfirmation(guess, query);
      html += `<div class="response-query-tag" style="background: rgba(219, 39, 119, 0.1); color: #DB2777;">โหมด: ศัพท์เฉพาะทางและศัพท์บัญญัติ (Specialized Terminology)</div>`;

      if (data.entry) {
        const entry = data.entry;
        html += `<div class="response-highlight-box" style="border-left-color: #DB2777;">
          <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap; margin-bottom: 6px;">
            <span style="font-size: 1.35rem; font-weight: 700; color: var(--text-primary);">${escapeHtml(entry.word)}</span>
            ${entry.pos ? `<span style="font-size: 0.95rem; color: #DB2777; font-weight: 500;">${escapeHtml(entry.pos)}</span>` : ''}
            ${entry.subject ? `<span style="padding: 2px 10px; border-radius: 999px; background: rgba(219, 39, 119, 0.1); color: #DB2777; font-size: 0.85rem; font-weight: 600;">สาขาวิชา: ${escapeHtml(entry.subject)}</span>` : ''}
            ${entry.source ? `<span style="font-size: 0.8rem; color: var(--text-muted); opacity: 0.75; margin-left: auto;">${escapeHtml(entry.source)}</span>` : ''}
          </div>
          <p style="font-size: 1.05rem; line-height: 1.5; color: var(--text-secondary);">${escapeHtml(entry.definition || '')}</p>
        </div>`;

        // Radial SVG graph
        if (data.related && data.related.length > 0) {
          html += `<h4 style="margin-top: 24px; color: var(--text-secondary); text-align: center;">แผนภาพเครือข่ายความสัมพันธ์คำศัพท์</h4>`;
          html += buildGraphSvg(data);
        }

        // Related Technical Terms
        if (data.related && data.related.length > 0) {
          html += `<h4 style="margin-top: 20px; color: var(--text-secondary);">ศัพท์ที่เกี่ยวโยงในสาขาวิชา</h4>`;
          html += `<ul style="list-style: none; padding: 0; margin-top: 10px;">`;
          data.related.forEach(r => {
            html += `<li style="padding: 12px 0; border-bottom: 1px dashed var(--glass-border);">
              <strong style="color: #DB2777; font-size: 1.25rem; cursor: pointer;" class="rw" data-word="${escapeHtml(r.word)}">${escapeHtml(r.word)}</strong>
              <span style="font-size: 0.95rem; color: var(--text-muted); margin-left: 10px;">ความสัมพันธ์: ${typeof r.score === 'number' ? r.score.toFixed(2) : r.score}</span>
              <div style="font-size: 1.05rem; line-height: 1.6; color: var(--text-primary); margin-top: 6px; padding-left: 14px; border-left: 3px solid #DB2777;">`;
            (r.path || []).forEach(p => {
              html += `<div style="padding: 2px 0;">↳ ${escapeHtml(p)}</div>`;
            });
            html += `</div></li>`;
          });
          html += `</ul>`;
        }

      } else {
        html += `<div class="response-highlight-box" style="border-left-color: #DB2777;">
          <p>ไม่พบศัพท์เฉพาะทางสำหรับ "${escapeHtml(query)}" ในคลังศัพท์บัญญัติ</p>
        </div>`;
      }

      responseBody.innerHTML = html;
      attachIntentChipListeners();

      responseBody.querySelectorAll('.rw').forEach(el => {
        el.addEventListener('click', () => {
          const w = el.getAttribute('data-word');
          if (w) {
            searchInput.value = w;
            executeSpecializedSearch(w, null);
            window.scrollTo({ top: 0, behavior: "smooth" });
          }
        });
      });

    } catch (e) {
      responseBody.innerHTML = `<p style="color: var(--accent-magenta);">เกิดข้อผิดพลาดในการดึงข้อมูลศัพท์เฉพาะทาง: ${escapeHtml(e.message)}</p>`;
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

