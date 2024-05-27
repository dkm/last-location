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
      document.getElementById('user_id').innerHTML = userInfo.id;
      document.getElementById('user_name').innerHTML = userInfo.name;
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

function load_last_position(uniq_url) {
  fetch ("/api/get_last_location?" + new URLSearchParams({
    url: uniq_url,
    count: 10,
  })).then(response => {
      return response.json();
  })
  .then(allUserInfo => {
    let userInfo = allUserInfo[0];
    let prevInfo = allUserInfo.slice(1);

    document.getElementById('user_id').innerHTML = userInfo.user_id;
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
        let pdate = new Date(allUserInfo[p].device_timestamp).toTimeString();
        detailed_coords_ol.innerHTML = detailed_coords_ol.innerHTML + `<li>${allUserInfo[p].lat} ${allUserInfo[p].lon} -- ${pdate}</li>`
      }
    }

    L.marker([userInfo.lat, userInfo.lon]).addTo(map)
     .bindPopup('Last position:<br>' + convert_from_epoch(userInfo.device_timestamp))
     .openPopup()
 })
}
