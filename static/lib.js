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


function load_last_position(pilot_id, elt_id) {
  fetch ("/api/json/" + pilot_id)
    .then(response => {
      return response.json();
    })
  .then(pilotInfo => {
    global_info = pilotInfo;

    document.getElementById('pilot_id').innerHTML = pilotInfo.id;
    document.getElementById('pilot_accuracy').innerHTML = pilotInfo.accuracy;
    document.getElementById('pilot_provider').innerHTML = pilotInfo.loc_provider;
    document.getElementById('pilot_latitude').innerHTML = pilotInfo.lat;
    document.getElementById('pilot_longitude').innerHTML = pilotInfo.lon;
    document.getElementById('pilot_altitude').innerHTML = pilotInfo.altitude;
    document.getElementById('pilot_speed').innerHTML = pilotInfo.speed;
    document.getElementById('pilot_heading').innerHTML = pilotInfo.direction;
    document.getElementById('pilot_battery').innerHTML = pilotInfo.battery;

    document.getElementById('pilot_timestamp').innerHTML = convert_from_epoch(pilotInfo.ts);

    let a = document.getElementById('osm_link');
    a.href="https://www.openstreetmap.org/?mlat=" + global_info.lat + "&mlon=" + global_info.lon;
  })
}
