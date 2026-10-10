# World display

The View menu above the scene switches between physical surface/water and measured field maps. Maps color the existing terrain with an unlit material, so lighting does not obscure differences. Fuel, minerals, detritus and waste are mass per surface area; water is depth in model units; light is relative availability. Each map uses its own logarithmic maximum and palette, shared with the Ecology inspector. The legend reports the range. Maps are interpolations of the existing, capped field grid, not extra simulated resolution. Missing fields are identified rather than invented.

Physical surface water uses the terrain's topology, sampled ground plus relief plus water depth, with a small display-only separation and polygon depth offset. Only triangles whose three vertices have more than 0.015 units of water are rendered. Thinner films remain in the simulation and the water-depth map but are not drawn as an overlapping transparent ocean. The mesh stays simple and rough; there are no reflections, bloom, waves or additional simulation work. Water is hidden while a field map replaces surface colors.

Field maps replace the former sparse point cloud. Water uses one reused index buffer for its wet-triangle mask. Planar terrain tessellation is capped with the field grid, matching the spherical display's bounded geometry. Surface updates remain throttled to once per second; choosing a layer refreshes the current snapshot immediately, including while paused.

Validation for this change is the TypeScript and production build only. No simulation, benchmark, or browser run was performed; visual runtime verification is left to the user.
