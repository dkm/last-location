// Ugly routine to convert the serialized JSON that seems to be encoded in
// Ordinal form.
function convert_timestamp(json_date) {
   let d = new Date();
   d.setFullYear(json_date[0], 0, 1);
   d.setHours(json_date[2], json_date[3], json_date[4])

   let b = new Date(d.getTime() + (json_date[1]-1) * 24 * 3600 * 1000);
   return b.toUTCString();
}

function convert_from_epoch(epoch){
  var d = new Date(0);
  d.setUTCSeconds(epoch);
  return d;
}

function wrap_empty_string(s) {
  if (!s) {return "Not available";}
  else {return s};
}

function create_new_user(name, url) {
  const data = {
    req_url: url,
    name: name,
  };
  const searchParams = new URLSearchParams(data);

  console.debug(name, url);
  console.debug(data);
  fetch ("/api/new", {
    method: "POST",
    mode: "cors",
    cache: "no-cache",
    body: searchParams,
  }).then(response => {
    return response.json();
  })
    .then(userInfo => {
      document.getElementById('log_id').innerHTML = userInfo.id;
      document.getElementById('user_priv_token').innerHTML = userInfo.priv_token;
      document.getElementById('user_unique_url').innerHTML =  userInfo.unique_url;

      document.getElementById('gpslogger').innerHTML = 'lat=%LAT&' +
        'lon=%LON&' +
        'accuracy=%ACC&' +
        'priv_token=' + userInfo.priv_token + '&' +
        'speed=%SPD&' +
        'direction=%DIR&' +
        'loc_provider=%PROV&' +
        'device_timestamp=%TIMESTAMP&' +
        'battery=%BATT&' +
        'altitude=%ALT';
      document.getElementById('gpslogger').style.display = "block";
    });
}

function display_positions(allUserInfo){
    let userInfo = allUserInfo[0];
    let prevInfo = allUserInfo.slice(1);

    document.getElementById('log_id').innerHTML = userInfo.log_id;
    document.getElementById('user_location_id').innerHTML = userInfo.id;
    document.getElementById('user_latitude').innerHTML = userInfo.lat;
    document.getElementById('user_longitude').innerHTML = userInfo.lon;

    document.getElementById('user_accuracy').innerHTML = wrap_empty_string(userInfo.accuracy);
    document.getElementById('user_provider').innerHTML = wrap_empty_string(userInfo.loc_provider);
    document.getElementById('user_altitude').innerHTML = wrap_empty_string(userInfo.altitude);
    document.getElementById('user_speed').innerHTML = wrap_empty_string(userInfo.speed);
    document.getElementById('user_heading').innerHTML = wrap_empty_string(userInfo.direction);
    document.getElementById('user_battery').innerHTML = wrap_empty_string(userInfo.battery);

    document.getElementById('user_device_timestamp').innerHTML = convert_from_epoch(userInfo.device_timestamp);
    document.getElementById('user_server_timestamp').innerHTML = convert_from_epoch(userInfo.server_timestamp);

    let a = document.getElementById('osm_link');
    a.href="https://www.openstreetmap.org/?mlat=" + userInfo.lat + "&mlon=" + userInfo.lon;

    var map = L.map('map').setView([userInfo.lat, userInfo.lon], 13);

    L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
      attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
    }).addTo(map);

    if (userInfo.accuracy){
       L.circle([userInfo.lat, userInfo.lon], {
         color: 'red',
         fillColor: '#f03',
         fillOpacity: 0.5,
         radius: userInfo.accuracy,
      }).addTo(map);
    }

    let detailed_coords_ol = document.getElementById('detailed-coords');

    if (prevInfo) {
      let prevPoints = allUserInfo.map((ui) => [ui.lat, ui.lon]);

      var polyline = L.polyline(prevPoints, {
        color: 'red',
        opacity: 0.5,
      }).addTo(map);

      for (p in allUserInfo) {
        let pdate = new Date(allUserInfo[p].device_timestamp * 1000).toISOString();
        let tooltip = `<ul><li>lat:${allUserInfo[p].lat}</li><li>lon: ${allUserInfo[p].lon}</li><li>time: ${pdate}</li>`
        if (allUserInfo[p].speed) {
          tooltip += `<li>speed: ${allUserInfo[p].speed}</li>`
        }
        tooltip += "</ul>"
        detailed_coords_ol.innerHTML = detailed_coords_ol.innerHTML + `<li>${allUserInfo[p].lat} ${allUserInfo[p].lon} -- ${pdate}</li>`

        L.circle([allUserInfo[p].lat, allUserInfo[p].lon], {
          color: 'red',
          fillColor: '#f03',
          fillOpacity: 0.5,
          radius: allUserInfo[p].accuracy,
        }).bindTooltip(tooltip).addTo(map);
      }
    }

    L.marker([userInfo.lat, userInfo.lon]).addTo(map)
     .bindPopup('Last position:<br>' + new Date(userInfo.device_timestamp * 1000).toISOString())
     .openPopup()
}

async function load_last_position(uniq_url) {
  var resp = await fetch ("/api/get_last_location?" + new URLSearchParams({
    url: uniq_url,
    count: 20,
    cut_last_segment: true,
  }));

  var positions = await resp.json();
  display_positions(positions);
}

const hex_decode = (string) => {
    const uint8array = new Uint8Array(Math.ceil(string.length / 2));
    for (let i = 0; i < string.length;)
        uint8array[i / 2] = Number.parseInt(string.slice(i, i += 2), 16);
    return uint8array;
}

async function load_last_position_sec(uniq_url) {
  var str_key = window.location.hash.substring(1);
  let byte_key = hex_decode(str_key);

  let key = await window.crypto.subtle.importKey(
    "raw",
    byte_key,
    {
	    name: "AES-GCM",
	  },
    false,
	  ["decrypt"]
  );

  const response = await fetch ("/api/s/get_last_location?" + new URLSearchParams({
    url: uniq_url,
    count: 20,
    cut_last_segment: true,
  }));

  const res_json = await response.json();

  var all_res = new Array();

  for (u in res_json) {
    let all_data = new Uint8Array(res_json[u].data);

    let iv = all_data.slice(0, 12); // 96-bits IV
    let buf = all_data.slice(12);

    let plain_text_deciphered =  await window.crypto.subtle.decrypt(
      {
	      name: "AES-GCM",
	      iv: iv,
	      tagLength: 128,
	    },
      key,
      buf,
    );

    var decoder = new TextDecoder("utf-8");
    var dec_json = JSON.parse(decoder.decode(plain_text_deciphered));
    all_res.push(dec_json);
  }
  display_positions(all_res);
}
