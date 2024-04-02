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


function load_last_position(user_id, elt_id) {
  fetch ("/api/json/" + user_id)
    .then(response => {
      return response.json();
    })
  .then(userInfo => {
    global_info = userInfo;

    document.getElementById('user_id').innerHTML = userInfo.user_id;
    document.getElementById('user_location_id').innerHTML = userInfo.id;
    document.getElementById('user_accuracy').innerHTML = userInfo.accuracy;
    document.getElementById('user_provider').innerHTML = userInfo.loc_provider;
    document.getElementById('user_latitude').innerHTML = userInfo.lat;
    document.getElementById('user_longitude').innerHTML = userInfo.lon;
    document.getElementById('user_altitude').innerHTML = userInfo.altitude;
    document.getElementById('user_speed').innerHTML = userInfo.speed;
    document.getElementById('user_heading').innerHTML = userInfo.direction;
    document.getElementById('user_battery').innerHTML = userInfo.battery;

    document.getElementById('user_device_timestamp').innerHTML = convert_from_epoch(userInfo.device_timestamp);
    document.getElementById('user_server_timestamp').innerHTML = convert_from_epoch(userInfo.server_timestamp);

    let a = document.getElementById('osm_link');
    a.href="https://www.openstreetmap.org/?mlat=" + global_info.lat + "&mlon=" + global_info.lon;

    var map = L.map('map').setView([userInfo.lat, userInfo.lon], 13);

    L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
      attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
    }).addTo(map);

    L.marker([userInfo.lat, userInfo.lon]).addTo(map)
     .bindPopup('Last position:<br>' + convert_from_epoch(userInfo.device_timestamp))
     .openPopup()
 })
}
