#!/usr/bin/env python3
"""
preview.py — Bone = HTML, PNG = CSS.

19 tulang adalah DRIVER. Setiap SCML part menempel ke tulang
dengan offset (dx, dy). Bone angles default dari SCML FK.

Kunci:
  q/ESC   keluar      s       save
  TAB     cycle       b       cycle bone
  ←→↑↓   nudge        +/-    rotate
  r       reset part
"""

import json, math, sys
from pathlib import Path
import xml.etree.ElementTree as ET
import numpy as np
from PIL import Image

ROOT = Path(__file__).parent
CONFIG = ROOT / "config_preview.json"

CHARACTERS = {
    "terrorist_1":  ("craftpix-485144-2d-game-terrorists-character-free-sprites-sheets", "terrorist_1"),
    "terrorist_2":  ("craftpix-485144-2d-game-terrorists-character-free-sprites-sheets", "terrorist_2"),
    "terrorist_3":  ("craftpix-485144-2d-game-terrorists-character-free-sprites-sheets", "terrorist_3"),
    "police_1":     ("craftpix-543219-2d-game-police-character-free-sprite-sheets", "1"),
    "police_2":     ("craftpix-543219-2d-game-police-character-free-sprite-sheets", "2"),
    "police_3":     ("craftpix-543219-2d-game-police-character-free-sprite-sheets", "3"),
    "chibi_summer": ("craftpix-955440-2d-game-chibi-boy-free-character-sprite-sheet", "summer"),
    "chibi_autumn": ("craftpix-955440-2d-game-chibi-boy-free-character-sprite-sheet", "autumn"),
    "chibi_winter": ("craftpix-955440-2d-game-chibi-boy-free-character-sprite-sheet", "winter"),
}

BONE_LABELS = [
    "torso", "neck",
    "clavicle_l", "clavicle_r",
    "upper_arm_l", "forearm_l", "hand_l",
    "upper_arm_r", "forearm_r", "hand_r",
    "hip_l", "hip_r",
    "upper_leg_l", "lower_leg_l", "foot_l",
    "upper_leg_r", "lower_leg_r", "foot_r",
    "head",
]

# Bone angles to extract from SCML FK
# 19-bone angle key → (SCML bone_ref id, attribute, sign, offset)
# angle_key: which 19-bone angle to set
# scml_br_id: which SCML bone_ref id to read from
# attr: 'local_angle' or 'world_angle'
# sign: +1 or -1 for direction
# offset: degrees to add
# Actually simpler: I'll just read SCML bone world angles and map them.
# The 19-bone "spine_lower", "body_tilt" angles are more complex.

# ── SCML parser ─────────────────────────────────────────────
def parse_scml(text):
    root = ET.fromstring(text)
    folders = []
    for fel in root.findall("folder"):
        files = [{"id":int(fe.get("id",0)),"name":fe.get("name",""),
                  "w":float(fe.get("width",0)),"h":float(fe.get("height",0)),
                  "px":float(fe.get("pivot_x",0)),"py":float(fe.get("pivot_y",0))}
                 for fe in fel.findall("file")]
        folders.append({"id":int(fel.get("id",0)),"name":fel.get("name",""),"files":files})
    entities = []
    for eel in root.findall("entity"):
        anims = []
        for ael in eel.findall("animation"):
            bkf, okf = {}, {}
            mk = []
            ml = ael.find("mainline")
            if ml is not None:
                for ke in ml.findall("key"):
                    br = [{"id":int(c.get("id")),"tl":int(c.get("timeline")),"k":int(c.get("key")),
                           "par":int(c.get("parent")) if c.get("parent") is not None else None}
                          for c in ke if c.tag=="bone_ref"]
                    or_ = [{"id":int(c.get("id")),"tl":int(c.get("timeline")),"k":int(c.get("key")),
                            "par":int(c.get("parent")) if c.get("parent") is not None else None,
                            "z":int(c.get("z_index",0))}
                           for c in ke if c.tag=="object_ref"]
                    mk.append({"t":int(ke.get("time",0)),"br":br,"or":or_})
            for tl in ael.findall("timeline"):
                tid = int(tl.get("id")); is_bone = tl.get("object_type","")=="bone"
                for ke in tl.findall("key"):
                    kt = int(ke.get("time",0))
                    if is_bone:
                        be = ke.find("bone")
                        if be is not None:
                            bkf.setdefault(tid,[]).append({"t":kt,"x":float(be.get("x",0)),
                                "y":float(be.get("y",0)),"a":float(be.get("angle",0)),
                                "sx":float(be.get("scale_x",1)),"sy":float(be.get("scale_y",1))})
                    else:
                        oe = ke.find("object")
                        if oe is not None:
                            okf.setdefault(tid,[]).append({"t":kt,"f":int(oe.get("folder")),
                                "fl":int(oe.get("file")),"x":float(oe.get("x",0)),"y":float(oe.get("y",0)),
                                "a":float(oe.get("angle",0)),"al":float(oe.get("a",1)),
                            "sx":float(oe.get("scale_x",1)),"sy":float(oe.get("scale_y",1))})
            anims.append({"name":ael.get("name"),"len":int(ael.get("length")),"bkf":bkf,"okf":okf,"mk":mk})
        entities.append({"name":eel.get("name"),"anims":anims})
    return {"folders":folders,"entities":entities}

def get_file(data, fid, foid):
    for f in data["folders"]:
        if f["id"]==fid:
            for ff in f["files"]:
                if ff["id"]==foid: return ff
    return None

# ── Load PNG ────────────────────────────────────────────────
def load_png(data, fid, foid):
    fi = get_file(data, fid, foid)
    if fi is None: return None
    path = data["_path"]
    for f in data["folders"]:
        if f["id"]==fid and f["name"]:
            p = path / f["name"] / fi["name"]
            if p.exists(): return Image.open(p).convert("RGBA")
            p = path / f["name"] / fi["name"].split("/")[-1]
            if p.exists(): return Image.open(p).convert("RGBA")
    p = path / fi["name"]
    if p.exists(): return Image.open(p).convert("RGBA")
    return None

# ── 19-bone skeleton ────────────────────────────────────────
H = {"HR":0.100,"NK":0.040,"TO":0.260,"SW":0.120,"HW":0.080,
     "UA":0.150,"FA":0.130,"HD":0.060,"UL":0.220,"LL":0.200,"FT":0.090}

def skeleton(angles={}):
    a = lambda k: angles.get(k,0)
    bt = a("body_tilt")
    hx, hy = 0, 0
    sa = math.radians(90+bt+a("spine_lower"))
    sc, ss = math.cos(sa), math.sin(sa)
    ttx = hx + H["TO"]*sc; tty = hy + H["TO"]*ss
    na = sa + math.radians(a("neck"))
    ntx = ttx + H["NK"]*math.cos(na); nty = tty + H["NK"]*math.sin(na)
    ha = na + math.radians(a("head_turn"))
    htx = ntx + 2*H["HR"]*math.cos(ha); hty = nty + 2*H["HR"]*math.sin(ha)
    pl = sa - math.pi/2; pr = sa + math.pi/2
    slx = ttx + H["SW"]*math.cos(pl); sly = tty + H["SW"]*math.sin(pl)
    srx = ttx + H["SW"]*math.cos(pr); sry = tty + H["SW"]*math.sin(pr)
    ual = math.radians(-90+bt+a("shoulder_l"))
    elx = slx + H["UA"]*math.cos(ual); ely = sly + H["UA"]*math.sin(ual)
    fal = ual + math.radians(a("elbow_l"))
    wlx = elx + H["FA"]*math.cos(fal); wly = ely + H["FA"]*math.sin(fal)
    hal = fal + math.radians(a("wrist_l"))
    hlx = wlx + H["HD"]*math.cos(hal); hly = wly + H["HD"]*math.sin(hal)
    uar = math.radians(-90+bt+a("shoulder_r"))
    erx = srx + H["UA"]*math.cos(uar); ery = sry + H["UA"]*math.sin(uar)
    far = uar + math.radians(a("elbow_r"))
    wrx = erx + H["FA"]*math.cos(far); wry = ery + H["FA"]*math.sin(far)
    har = far + math.radians(a("wrist_r"))
    hrx = wrx + H["HD"]*math.cos(har); hry = wry + H["HD"]*math.sin(har)
    hlx_ = hx + H["HW"]*math.cos(pl); hly_ = hy + H["HW"]*math.sin(pl)
    hrx_ = hx + H["HW"]*math.cos(pr); hry_ = hy + H["HW"]*math.sin(pr)
    ull = math.radians(-90+bt+a("hip_l"))
    klx = hlx_ + H["UL"]*math.cos(ull); kly = hly_ + H["UL"]*math.sin(ull)
    lll = ull + math.radians(a("knee_l"))
    alx = klx + H["LL"]*math.cos(lll); aly = kly + H["LL"]*math.sin(lll)
    ftl = lll - math.pi/2 + math.radians(a("ankle_l"))
    flx = alx + H["FT"]*math.cos(ftl); fly = aly + H["FT"]*math.sin(ftl)
    ulr = math.radians(-90+bt+a("hip_r"))
    krx = hrx_ + H["UL"]*math.cos(ulr); kry = hry_ + H["UL"]*math.sin(ulr)
    llr = ulr + math.radians(a("knee_r"))
    arx = krx + H["LL"]*math.cos(llr); ary = kry + H["LL"]*math.sin(llr)
    ftr = llr + math.pi/2 + math.radians(a("ankle_r"))
    frx = arx + H["FT"]*math.cos(ftr); fry = ary + H["FT"]*math.sin(ftr)
    segs = [
        ("torso",hx,hy,ttx,tty),("neck",ttx,tty,ntx,nty),
        ("clavicle_l",ttx,tty,slx,sly),("clavicle_r",ttx,tty,srx,sry),
        ("upper_arm_l",slx,sly,elx,ely),("forearm_l",elx,ely,wlx,wly),("hand_l",wlx,wly,hlx,hly),
        ("upper_arm_r",srx,sry,erx,ery),("forearm_r",erx,ery,wrx,wry),("hand_r",wrx,wry,hrx,hry),
        ("hip_l",hx,hy,hlx_,hly_),("hip_r",hx,hy,hrx_,hry_),
        ("upper_leg_l",hlx_,hly_,klx,kly),("lower_leg_l",klx,kly,alx,aly),("foot_l",alx,aly,flx,fly),
        ("upper_leg_r",hrx_,hry_,krx,kry),("lower_leg_r",krx,kry,arx,ary),("foot_r",arx,ary,frx,fry),
        ("head",ntx,nty,htx,hty),
    ]
    return {l:{"x1":x1,"y1":y1,"x2":x2,"y2":y2,"dx":x2-x1,"dy":y2-y1,
               "angle":math.degrees(math.atan2(y2-y1,x2-x1)),
               "len":math.hypot(x2-x1,y2-y1)} for l,x1,y1,x2,y2 in segs}

# ── Part-to-bone mapping ────────────────────────────────────
PART2BONE = {
    "body":"torso","neck":"neck","head":"head",
    "left_shoulder":"upper_arm_l","right_shoulder":"upper_arm_r",
    "left_forearm":"forearm_l","right_forearm":"forearm_r",
    "left_arm":"hand_l","right_arm":"hand_r",
    "left_hip":"upper_leg_l","right_hip":"upper_leg_r",
    "left_leg":"lower_leg_l","right_leg":"lower_leg_r",
    "pants":"torso","skirt":"torso","shadow":None,
}
def p2b(name):
    n = name.lower().replace(".png","")
    if n in PART2BONE: return PART2BONE[n]
    for k,v in PART2BONE.items():
        if k in n or n in k: return v
    return None

# ── Bone angle extraction from SCML ─────────────────────────
# ── SCML bone_ref id → 19-bone label mapping ──────────────────
# Bone tree (terrorist skeleton):
#   br0 = root (pelvis/hip-base)
#   br1 = spine / lower-torso→neck-base (child of br0)
#   br2 = neck→head (child of br1)
#   br3 = shoulder_l (child of br0)
#   br4 = elbow_l (child of br3)
#   br5 = wrist_l (child of br4)
#   br6 = shoulder_r (child of br0)
#   br7 = elbow_r (child of br6)
#   br8 = wrist_r (child of br7)
#   br9 = hip_l (child of br0)
#   br10 = knee_l (child of br9)
#   br11 = hip_r (child of br0)
#   br12 = knee_r (child of br11)
#   br13 = extra (skirt/pants) (child of br0)
SCML_BR_TO_BONE19 = {
    0: "torso",
    1: "torso",      # spine attaches to same torso anchor
    2: "head",       # neck→head (head object sits here)
    3: "upper_arm_l",4:"forearm_l",5:"hand_l",
    6: "upper_arm_r",7:"forearm_r",8:"hand_r",
    9: "upper_leg_l",10:"lower_leg_l",
    11:"upper_leg_r",12:"lower_leg_r",
    13:"torso",      # skirt/pants attach to torso root
}

# Override per-part when the parent bone implies a finer 19-bone choice based on file name
# (e.g. 'right_arm.png' parent=br8 should map to hand_r, while 'left_arm.png' parent=br5 → hand_l).
def bone_for_object(fname, parent_bone_ref):
    """Determine which of the 19 bones owns this part. Prefer mapping by name; fall back to parent bone_ref."""
    n = fname.lower().replace(".png","")
    # Direct name → bone map
    P2B = {
        "body":"torso","neck":"neck","head":"head",
        "left_shoulder":"upper_arm_l","right_shoulder":"upper_arm_r",
        "left_forearm":"forearm_l","right_forearm":"forearm_r",
        "left_arm":"hand_l","right_arm":"hand_r",
        "left_hip":"upper_leg_l","right_hip":"upper_leg_r",
        "left_leg":"lower_leg_l","right_leg":"lower_leg_r",
        "left_leg_1":"lower_leg_l","right_leg_1":"lower_leg_r",
        "pants":"torso","skirt":"torso","shadow":None,
    }
    if n in P2B: return P2B[n]
    # Fuzzy matches
    for k,v in P2B.items():
        if k in n or n in k: return v
    if parent_bone_ref is not None:
        return SCML_BR_TO_BONE19.get(parent_bone_ref, "torso")
    return "torso"

def extract_scml_angles(data):
    """Extract default bone angles from SCML FK at t=0 as LOCAL bone angles
    (relative to parent), suitable for the 19-bone FK model."""
    if not data or not data["entities"]: return {}
    ent = data["entities"][0]
    if not ent["anims"]: return {}
    anim = ent["anims"][0]
    mk = anim["mk"][0] if anim["mk"] else None
    if mk is None: return {}
    angles = {}
    # Read local bone angles keyed by bone_ref id 0..N
    b_loc = {}
    for br in mk["br"]:
        kfs = anim["bkf"].get(br["tl"], [])
        kf = next((k for k in kfs if k["t"]<=0), kfs[0] if kfs else None)
        if kf: b_loc[br["id"]] = kf["a"]
        else:  b_loc[br["id"]] = 0
    # bone_000 root — overall body tilt in Spriter (CW from +x); vertical up would be 90 CCW = 90 hijau
    # our 19-bone 'body_tilt' is measured around vertical-up axis; default 0 means standing up.
    # Spriter root angle close to 84.5° (nearly vertical-up), so body_tilt ≈ b0_angle - 90.
    angles["body_tilt"] = b_loc.get(0, 0) - 90
    angles["spine_lower"] = b_loc.get(1, 0)             # bend of spine relative to root
    # neck angle (br2) already includes spine + neck turn; use its local angle
    angles["neck"] = b_loc.get(2, 0)
    angles["head_turn"] = 0
    angles["shoulder_l"] = b_loc.get(3, 0)
    angles["elbow_l"]    = b_loc.get(4, 0)
    angles["wrist_l"]    = b_loc.get(5, 0)
    angles["shoulder_r"] = b_loc.get(6, 0)
    angles["elbow_r"]    = b_loc.get(7, 0)
    angles["wrist_r"]    = b_loc.get(8, 0)
    angles["hip_l"]      = b_loc.get(9, 0)
    angles["knee_l"]     = b_loc.get(10, 0)
    angles["hip_r"]      = b_loc.get(11, 0)
    angles["knee_r"]     = b_loc.get(12, 0)
    print(f"[scml_angles] {angles}")
    return angles

def compute_fk_world_positions(data):
    """Compute world positions + angles for bones and objects from SCML FK at t=0.
    Spriter convention: angle in degrees, +x right, +y UP (math-y), angle measured CW from +x.
    For object rendering we convert to a display convention by flipping y.
    """
    if not data or not data["entities"]: return {}, {}
    ent = data["entities"][0]
    if not ent["anims"]: return {}, {}
    anim = ent["anims"][0]
    mk = anim["mk"][0] if anim["mk"] else None
    if mk is None: return {}, {}
    b_loc = {}
    for br in mk["br"]:
        kfs = anim["bkf"].get(br["tl"], [])
        kf = next((k for k in kfs if k["t"]<=0), kfs[0] if kfs else {"x":0,"y":0,"a":0,"sx":1,"sy":1})
        b_loc[br["id"]] = (kf["x"], kf["y"], kf["a"], kf.get("sx",1), kf.get("sy",1))
    # Bone FK — parent scale applied to position; angle sign flipped when
    # parent has odd number of negative scales (Spriter convention for
    # child local angle in a flipped bone's coordinate system).
    b_world = {}
    for br in mk["br"]:
        lx, ly, la, lsx, lsy = b_loc.get(br["id"], (0,0,0,1,1))
        pid = br["par"]
        if pid is not None and pid in b_world:
            pw = b_world[pid]
            rad = math.radians(pw[2]); c, s = math.cos(rad), math.sin(rad)
            pwsx, pwsy = pw[3], pw[4]
            wx = pw[0] + c * lx * pwsx - s * ly * pwsy
            wy = pw[1] + s * lx * pwsx + c * ly * pwsy
            scale_sign = -1 if pwsx * pwsy < 0 else 1
            wa = pw[2] + la * scale_sign
            wsx = pwsx * lsx
            wsy = pwsy * lsy
        else:
            wx, wy, wa, wsx, wsy = lx, ly, la, lsx, lsy
        b_world[br["id"]] = (wx, wy, wa, wsx, wsy)
    # Object FK — applies parent bone world scale to position; angle sign
    # flipped when parent has odd number of negative scales.
    o_world = {}
    for ore in mk["or"]:
        kfs = anim["okf"].get(ore["tl"], [])
        kf = next((k for k in kfs if k["t"]<=0), kfs[0] if kfs else None)
        if kf is None: continue
        lx, ly, la = kf["x"], kf["y"], kf["a"]
        osx, osy = kf.get("sx",1), kf.get("sy",1)
        pid = ore.get("par")
        wsx, wsy = 1.0, 1.0  # default world scale
        if pid is not None and pid in b_world:
            pw = b_world[pid]
            rad = math.radians(pw[2]); c, s = math.cos(rad), math.sin(rad)
            pwsx, pwsy = pw[3], pw[4]
            wx = pw[0] + c * lx * pwsx - s * ly * pwsy
            wy = pw[1] + s * lx * pwsx + c * ly * pwsy
            scale_sign = -1 if pwsx * pwsy < 0 else 1
            wa = pw[2] + la * scale_sign
            wsx = pwsx * osx
            wsy = pwsy * osy
        else:
            wx, wy, wa = lx, ly, la
            wsx, wsy = osx, osy
        fi = get_file(data, kf["f"], kf["fl"])
        if fi:
            o_world[fi["name"]] = {"x":wx,"y":wy,"angle":wa,
                                    "local_angle":la,
                                    "parent_bone_ref":pid,
                                    "file":fi,"or_id":ore.get("id"),
                                    "sx":wsx,"sy":wsy}
    return o_world, b_world

def auto_calibrate(data, angles):
    """Build offset map anchored to SCML FK world positions.
    PNG ditempatkan langsung di FK world position — initial offsets nol.
    Returns (offsets_dict, scml_h) where scml_h = character height in SCML units
    (used to normalize pixel FK coords into world units that match the 19-bone model).
    """
    obj_world, bone_world = compute_fk_world_positions(data)
    # Character height = vertical spread of object FK pivots (head object vs. legs).
    # Object pivots include head.png whose bottom is the tallest; bone pivots don't.
    if obj_world:
        ys = [pos["y"] for pos in obj_world.values()]
        scml_h = max(ys) - min(ys) if len(ys) > 1 else 1100.0
    else:
        scml_h = 1100.0
    scml_h = max(scml_h, 100.0)
    offsets = {}
    for fn, o in obj_world.items():
        parent_br = o.get("parent_bone_ref")
        bone = bone_for_object(fn, parent_br)
        if bone is None:
            offsets[fn] = {"bone":"", "dx":0, "dy":0, "angle":0}
        else:
            offsets[fn] = {"bone":bone, "dx":0, "dy":0, "angle":0}
    return offsets, scml_h

def fk_skeleton_overlay(data, scml_h):
    """Build a 19-bone-labelled skeleton from SCML bone FK world positions.
    Converts Spriter y-up pixel space to render space (y-up world units, same as 19-bone model).
    Maps each bone_ref id → connected segment; the 19-bone label is derived from the
    set of objects that attach to that bone (so structure is portable across rigs).
    """
    if not data or not data["entities"]: return []
    ent = data["entities"][0]
    if not ent["anims"]: return []
    anim = ent["anims"][0]
    mk = anim["mk"][0] if anim["mk"] else None
    if mk is None: return []
    obj_world, b_world = compute_fk_world_positions(data)
    sw = 1.0/scml_h

    # Build bone_ref id → 19-bone label via the object(s) attached to it.
    # Most parts have a filename that bone_for_object can label (body, head, left_leg...).
    # If a bone has no objects attached, fall back to its local order (root=torso).
    br_id_to_label = {}
    for ore in mk["or"]:
        # get the object's filename (via the animate-attached file)
        kfs = anim["okf"].get(ore["tl"], [])
        kf = next((k for k in kfs if k["t"]<=0), kfs[0] if kfs else None)
        if kf is None: continue
        fi = get_file(data, kf["f"], kf["fl"])
        if fi is None: continue
        fn = fi["name"]
        parent = ore["par"]
        label = bone_for_object(fn, parent)
        if parent is not None and label is not None:
            # Prefer a part that maps to a non-"torso" label (more specific)
            if parent not in br_id_to_label or br_id_to_label[parent] in ("torso", "extra"):
                br_id_to_label[parent] = label

    # Reuse default static map for any bone_ref not covered (esp. root=torso)
    for br in mk["br"]:
        if br["id"] not in br_id_to_label:
            br_id_to_label[br["id"]] = SCML_BR_TO_BONE19.get(br["id"], "extra")

    segs = []
    for br in mk["br"]:
        br_id = br["id"]; par_id = br["par"]
        if par_id is not None and par_id in b_world:
            pwx, pwy, *_ = b_world[par_id]
        else:
            # Root segment: parent is itself, just a point
            pwx, pwy, *_ = b_world[br_id]
        wx, wy, *_ = b_world[br_id]
        pwx_w, pwy_w = pwx*sw, pwy*sw
        wx_w, wy_w = wx*sw, wy*sw
        label = br_id_to_label.get(br_id, "extra")
        segs.append({"label":label, "x1":pwx_w, "y1":pwy_w, "x2":wx_w, "y2":wy_w})
    return segs

# ── Config ──────────────────────────────────────────────────
def load_cfg():
    if CONFIG.exists():
        try: c = json.loads(CONFIG.read_text())
        except: c = {}
        if c.get("v")==7: return c
        print(f"[config] migrate v{c.get('v',0)}→v7 — FK parent-scale positions + Rust-match angles")
        return {"v":7,"chars":{},"view":{"cx":0,"cy":0.5,"zoom":350}}
    return {"v":7,"chars":{},"view":{"cx":0,"cy":0.5,"zoom":350}}

def save_cfg(c):
    CONFIG.write_text(json.dumps(c, indent=2))
    print("[preview] saved")

# ── Load SCML ───────────────────────────────────────────────
def load_scml(skin):
    info = CHARACTERS.get(skin)
    if not info: return None
    pd, sub = info
    d = ROOT / pd / "scml" / sub
    if not d.exists(): return None
    fl = sorted(d.glob("*.scml"))
    if not fl: return None
    data = parse_scml(fl[0].read_text("utf-8"))
    data["_path"] = d
    return data

# ── Matplotlib ──────────────────────────────────────────────
import matplotlib
matplotlib.use("TkAgg")
import matplotlib.pyplot as plt

class App:
    def __init__(self, skin="terrorist_1"):
        self.skin = skin
        self.cfg = load_cfg()
        cc = self.cfg["chars"].setdefault(skin, {})
        self.view = self.cfg.get("view", {"cx":0,"cy":0.5,"zoom":350})

        self.data = load_scml(skin)
        self.parts = []

        # 1. Load or extract default bone angles (local angles)
        self.angles = cc.get("angles", {})
        if not self.angles and self.data:
            self.angles = extract_scml_angles(self.data)
            print(f"[angles] using SCML defaults: {self.angles}")

        # 2. Load or auto-calibrate part offsets
        self.parts_cfg = cc.get("parts", {})
        self.scml_h = 1100.0
        need_recalibrate = (not self.parts_cfg and self.data)
        if need_recalibrate:
            print("[calibrate] no saved offsets — auto-calibrating from SCML FK...")
            offsets, self.scml_h = auto_calibrate(self.data, self.angles)
            self.parts_cfg = offsets
            cc["parts"] = self.parts_cfg
            cc["angles"] = self.angles
            save_cfg(self.cfg)
        elif self.data:
            # Reuse scml_h computed from object FK (character height including head png)
            obj_world, bone_world = compute_fk_world_positions(self.data)
            if obj_world:
                ys = [pos["y"] for pos in obj_world.values()]
                self.scml_h = max(max(ys)-min(ys), 100.0) if len(ys) > 1 else 1100.0

        # 3. Build parts
        self._build_parts()

        self.sel = None
        self.fig, self.ax = plt.subplots(figsize=(14,10))
        self.fig.canvas.mpl_connect("key_press_event", self._key)
        self.fig.canvas.mpl_connect("button_press_event", self._click)
        self.fig.canvas.mpl_connect("scroll_event", self._scroll)
        self._render()

    def _build_parts(self):
        """Build parts anchored to SCML FK world position (the source of truth).
        Each part is placed at (fk_wx, fk_wy) in world units and rotated by its
        FK world angle. User offsets (dx, dy, angle) fine-tune placement.
        Spriter stores angles in degrees CW from +x in a y-UP world.
        For matplotlib display we flip y to y-DOWN-equivalent world by negating
        angles and y — keeping the math consistent so PNG lands exactly where
        the bone drives it.
        """
        if not self.data or not self.data["entities"]: return
        ent = self.data["entities"][0]
        if not ent["anims"]: return
        anim = ent["anims"][0]
        mk = anim["mk"][0] if anim["mk"] else None
        if mk is None: return

        o_world, b_world = compute_fk_world_positions(self.data)
        # Cache rotated image once per part for default FK angle (so re-render is cheap)
        for ore in mk["or"]:
            kfs = anim["okf"].get(ore["tl"], [])
            kf = next((k for k in kfs if k["t"]<=0), kfs[0] if kfs else None)
            if kf is None: continue
            fi = get_file(self.data, kf["f"], kf["fl"])
            if fi is None: continue
            fn = fi["name"]
            png = load_png(self.data, kf["f"], kf["fl"])
            if png is None: continue

            fk_w = o_world.get(fn)
            if fk_w is None: continue
            # Bone label picked from object filename + parent bone_ref fallback
            parent_br = fk_w.get("parent_bone_ref")
            default_bone = bone_for_object(fn, parent_br) if bone_for_object(fn, parent_br) is not None else ""

            pc = self.parts_cfg.get(fn, {})
            if "bone" in pc:
                bone = pc.get("bone", default_bone) or ""
            else:
                bone = default_bone or ""
            dx = pc.get("dx", 0)
            dy = pc.get("dy", 0)
            angle_off = pc.get("angle", 0)

            self.parts.append([fn, png, fk_w["angle"], fi["px"], fi["py"],
                               bone, dx, dy, angle_off, kf, fk_w["x"], fk_w["y"],
                               fk_w.get("sx",1), fk_w.get("sy",1)])

    def _render(self):
        self.ax.clear()
        sw = 1.0/float(self.scml_h)

        # ── Draw skeleton FROM SCML FK bone world positions ──
        sk_segs = fk_skeleton_overlay(self.data, self.scml_h)
        majors = {"torso","neck","head","upper_arm_l","upper_arm_r",
                  "forearm_l","forearm_r","upper_leg_l","upper_leg_r",
                  "lower_leg_l","lower_leg_r","hip_l","hip_r"}
        for s in sk_segs:
            bx, by, ex, ey = s["x1"], s["y1"], s["x2"], s["y2"]
            # In matplotlib y-up is the natural axis — keep FK y-up as-is
            self.ax.plot([bx,ex],[by,ey],"r-",lw=2,alpha=0.5,zorder=0)
            self.ax.plot(bx,by,"ro",ms=3,alpha=0.6,zorder=1)
            if s["label"] in majors:
                mx, my = (bx+ex)/2, (by+ey)/2
                self.ax.text(mx,my,s["label"],fontsize=5,color="white",ha="center",
                             bbox=dict(boxstyle="round,pad=0.1",fc="black",alpha=0.5),zorder=1)

        # ── Draw parts at FK positions + user offsets ──
        for i, p in enumerate(self.parts):
            fn, png, fk_angle, px_, py_, bone, dx, dy, user_angle, kf, fk_x, fk_y, osx, osy = p

            # FK world position (SCML pixels → world units, y-up)
            fk_wx = fk_x * sw
            fk_wy = fk_y * sw
            wx = fk_wx + dx * sw
            wy = fk_wy + dy * sw

            # Spriter angle (CCW in y-up). PIL rotate(θ) is CCW in image y-down space,
            # which visually corresponds to CCW y-up θ = PIL rotate(-θ).
            angle = -fk_angle + user_angle

            # Apply object visual scale (resize + flip) before rotation.
            # After flipping, the pivot in UV space also flips.
            img = png
            w, h = img.size
            adj_px, adj_py = px_, py_
            if osx != 1 or osy != 1 or osx < 0 or osy < 0:
                new_w = max(int(w * abs(osx)), 1)
                new_h = max(int(h * abs(osy)), 1)
                if new_w != w or new_h != h:
                    img = img.resize((new_w, new_h), Image.BICUBIC)
                if osx < 0:
                    img = img.transpose(Image.FLIP_LEFT_RIGHT)
                    adj_px = 1.0 - adj_px
                if osy < 0:
                    img = img.transpose(Image.FLIP_TOP_BOTTOM)
                    adj_py = 1.0 - adj_py
                w, h = img.size

            # Rotate image around pivot (adjusted for flip)
            # adj_px: 0=left, 1=right   adj_py: 0=bottom, 1=top
            piv_px = adj_px * w
            piv_py = (1.0 - adj_py) * h

            if abs(angle) > 0.5:
                rad = math.radians(angle); c, s = math.cos(rad), math.sin(rad)
                cns = [(-piv_px,-piv_py),(w-piv_px,-piv_py),(w-piv_px,h-piv_py),(-piv_px,h-piv_py)]
                xs = [x*c - y*s for x,y in cns]
                ys = [x*s + y*c for x,y in cns]
                minx, miny = min(xs), min(ys)
                ox, oy = -minx, -miny
                # Use fast PIL built-in rotate then offset
                rot = img.rotate(angle, expand=True, resample=Image.BICUBIC)
                pox, poy = ox, oy
            else:
                rot = img; pox, poy = piv_px, piv_py

            rw, rh = rot.size
            # Part bounding box in world units with pivot anchored at (wx, wy)
            # Pixel (pox, poy) in the image (row 0 = top) maps to (wx, wy) in matplotlib y-up:
            #   left = wx - pox * sw    (x: pixel col pox → x coord wx)
            #   top  = wy + poy * sw    (y: pixel row poy → y coord wy, since row 0 is at top)
            left = wx - pox * sw
            right = left + rw * sw
            top = wy + poy * sw
            bottom = top - rh * sw

            selected = (i == self.sel)
            self.ax.imshow(rot, extent=[left,right,bottom,top],
                          alpha=0.95, zorder=10+i)

            if selected:
                rect = plt.Rectangle((left,bottom),right-left,top-bottom,
                                    fill=False,edgecolor="cyan",lw=2,zorder=1000)
                self.ax.add_patch(rect)
                # Connection to FK position
                self.ax.plot([fk_wx, wx], [fk_wy, wy], "y-", lw=1, alpha=0.7, zorder=999)
                self.ax.plot(fk_wx, fk_wy, "yo", ms=5, zorder=999)
                self.ax.plot(wx, wy, "co", ms=4, zorder=999)
                info = f"[{i}] {fn} → {bone}  dx={dx:.0f} dy={dy:.0f} angle={angle:.0f}  FK=({fk_x:.0f},{fk_y:.0f})"
                self.ax.set_title(info, fontsize=10, color="cyan")

        # Reference axis: ground at lowest object FK position (feet)
        if self.parts:
            fys = [(p[11]+p[7]) * sw for p in self.parts]
            ground = min(fys) if fys else 0
            cy_fys = [(p[11]+p[7]) * sw for p in self.parts]
            cy_w = (max(cy_fys) + min(cy_fys)) / 2 if cy_fys else 0.5
        else:
            ground = 0; cy_w = 0.5
        self.ax.axhline(y=ground, color="green", ls="--", alpha=0.3)
        cz = self.view["zoom"]/350; hs, vs = 2.0/cz, 1.5/cz
        # Auto-center view on character horizontal bbox (object FK pivots) and
        # vertical bbox (object y), so PNG always fits the viewport.
        if self.parts:
            cxs = [(p[10]+p[6]) * sw for p in self.parts]
            self.ax.set_xlim(min(cxs)-1.0/cz, max(cxs)+1.0/cz)
        else:
            self.ax.set_xlim(self.view["cx"]-hs, self.view["cx"]+hs)
        self.ax.set_ylim(cy_w - vs, cy_w + vs)
        self.ax.set_aspect("equal"); self.ax.grid(True, alpha=0.15)
        self.ax.set_title(f"{self.skin} | {len(self.parts)} parts | scml_h={self.scml_h:.0f} | TAB=cycle B=bone S=save", fontsize=9)
        self.fig.canvas.draw_idle()

    # ── Events ────────────────────────────────────────────────
    def _key(self, ev):
        if ev.key in ("q","escape"): plt.close()
        elif ev.key == "s": self._save()
        elif ev.key == "tab":
            if self.parts:
                self.sel = (self.sel+1)%len(self.parts) if self.sel is not None else 0
            self._render()
        elif ev.key == "b":
            if self.sel is not None:
                p = self.parts[self.sel]
                cur = p[5]
                if cur in BONE_LABELS:
                    idx = (BONE_LABELS.index(cur)+1)%len(BONE_LABELS)
                else:
                    idx = 0
                p[5] = BONE_LABELS[idx]; p[6]=0; p[7]=0
                self._render()
        elif ev.key == "r":
            if self.sel is not None:
                p = self.parts[self.sel]; p[6]=0; p[7]=0; p[8]=0; self._render()
        elif ev.key == "left": self._nudge(-1,0)
        elif ev.key == "right": self._nudge(1,0)
        elif ev.key == "up": self._nudge(0,1)
        elif ev.key == "down": self._nudge(0,-1)
        elif ev.key in ("=","+"): self._rot(5)
        elif ev.key == "-": self._rot(-5)

    def _nudge(self, dx, dy):
        if self.sel is None: return
        p = self.parts[self.sel]; p[6] += dx; p[7] += dy
        self._render()

    def _rot(self, deg):
        if self.sel is None: return
        p = self.parts[self.sel]; p[8] = (p[8] + deg) % 360
        self._render()

    def _click(self, ev):
        if ev.inaxes != self.ax: return
        sw = 1.0/self.scml_h
        wx, wy = ev.xdata, ev.ydata
        best, best_d = None, 999
        for i, p in enumerate(self.parts):
            fk_x, fk_y = p[10], p[11]
            px_ = (fk_x + p[6]) * sw
            py_ = (fk_y + p[7]) * sw
            d = math.hypot(px_-wx, py_-wy)
            if d < best_d: best_d = d; best = i
        if best is not None and best_d < 0.3:
            self.sel = best; self._render()

    def _scroll(self, ev):
        if self.sel is None: return
        self._rot(5 if ev.button=="up" else -5)

    def _save(self):
        cc = self.cfg["chars"].setdefault(self.skin, {})
        parts = {}
        for p in self.parts:
            fn = p[0]; bone = p[5]; dx = p[6]; dy = p[7]; ua = p[8]
            parts[fn] = {"bone":bone, "dx":dx, "dy":dy, "angle":ua}
        cc["parts"] = parts
        cc["angles"] = self.angles
        self.cfg["view"] = self.view
        save_cfg(self.cfg)

    def run(self):
        plt.show()

if __name__ == "__main__":
    skin = sys.argv[1] if len(sys.argv)>1 and sys.argv[1] in CHARACTERS else "terrorist_1"
    App(skin).run()
